use argparse::{ArgumentParser, StoreTrue, Store};
use num_complex::Complex;

use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access};//, State};

const FORMATUM: &str = "\"Freq[Hz]:phase[degree]:dutycycle[%]:amplitude(0..32767) Freq2:phase:dutycycle:...\"\n";

fn sound_init(pcm: &PCM, samplerate: u32, channelnum: u32) -> alsa::pcm::IO<i16> {
    // Set hardware parameters: 48000 Hz / Mono / 16 bit
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(channelnum).unwrap();
    hwp.set_rate(samplerate, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();
    let io = pcm.io_i16().unwrap();

    // Make sure we don't start the stream too early
    let hwp = pcm.hw_params_current().unwrap();
    let swp = pcm.sw_params_current().unwrap();
    swp.set_start_threshold(hwp.get_buffer_size().unwrap() - hwp.get_period_size().unwrap()).unwrap();
    pcm.sw_params(&swp).unwrap();
    return io;
}

enum Waveform {
    Sine,
    Square,
    Triangle,
    Nowave,
}

struct Sigparam {
    frequency: f64,
    phase: f64,
    dutycyclephase: f64,
    amplitude: f64,
}

fn sigtypedec(sigtypestr: &str, ch: &str) -> Waveform {
    match sigtypestr {
        "sine" => Waveform::Sine,
        "square" => Waveform::Square,
        "triangle" => Waveform::Triangle,
        _ => {eprintln!("{} csatornán nincs semmilyen hullámforma kiválasztva. Letiltva.", ch); Waveform::Nowave },
    }
}

fn sigstrdec(sigstr: &str) -> Vec<Sigparam> {
    let mut sigpvec: Vec<Sigparam> = Vec::new();
    if sigstr.len() == 0 {
        return sigpvec;
    }
    for sig in sigstr.split(' ') {
        let sv: Vec<&str> = sig.split(":").collect();
        if sv.len() != 4 {
            eprintln!("Hibás paraméterszám (sw.len = {}). Formátum: {}", sv.len(), FORMATUM);
        }

        let s = Sigparam {
            frequency: sv[0].parse().unwrap(),
            phase:     sv[1].parse::<f64>().unwrap() / 180.*::std::f64::consts::PI,
            dutycyclephase: sv[2].parse::<f64>().unwrap() / 100. * 2.*::std::f64::consts::PI,
            amplitude: sv[3].parse().unwrap(),
        };
        if s.frequency < 0. || s.frequency > 24000. {
            eprintln!("Freq range: 0..24000");
            ::std::process::exit(1);
        }

        if s.amplitude < 0. || s.amplitude > 32767. {
            eprintln!("Amp range: 0..32767");
            ::std::process::exit(1);
        }
        sigpvec.push(s);
    }
    return sigpvec;
}


fn main() {
    let mut verbose = false;
    let mut lefttypestr = String::new();
    let mut righttypestr = String::new();
    let mut leftsigstr = String::new();
    let mut rightsigstr = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Multisignal Frequency Generator.");
        ap.refer(&mut verbose).add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
        ap.refer(&mut lefttypestr).add_option(&["-L", "--lefttype"], Store, "sine,square,triangle").required();
        ap.refer(&mut righttypestr).add_option(&["-R", "--righttype"], Store, "sine,square,triangle");
        ap.refer(&mut leftsigstr).add_option(&["-l", "--leftsignal"], Store, FORMATUM).required();
        ap.refer(&mut rightsigstr).add_option(&["-r", "--rightsignal"], Store, FORMATUM);
        ap.parse_args_or_exit();
    }

    let lefttype = sigtypedec(&lefttypestr, "Left");
    let righttype = sigtypedec(&righttypestr, "Right");
    let mut sig = sigstrdec(&leftsigstr);
    let leftsiglen = sig.len();
    sig.append(&mut sigstrdec(&rightsigstr));


    let samplerate = 48000;
    // Open default playback device
    let pcm = PCM::new("default", Direction::Playback, false).unwrap();
    let io = sound_init(&pcm, 48000, 2);

    // Make the wave
    let mut buf = [0i16; 2*4096];
    let mut phasestepvec = Vec::new();
    let mut phvec = Vec::new();
    let mut ph_diffvec = Vec::new();
    for i in 0..sig.len() {
        phvec.push((Complex::i()*(sig[i].phase+::std::f64::consts::PI/2.)).exp()); // 90° shift
        //phvec.push(Complex::new(0., 1.));
        let phasestep = sig[i].frequency / samplerate as f64 * 2.*::std::f64::consts::PI;
        phasestepvec.push(phasestep);
        ph_diffvec.push((Complex::i()*phasestep).exp());
    }

    loop {
        match lefttype {
            Waveform::Sine => {
                for idx in 0..buf.len()/2 {
                    let mut signal = 0.;
                    for i in 0..leftsiglen {
                        signal += phvec[i].re * sig[i].amplitude;
                        phvec[i] *= ph_diffvec[i];
                    }
                    buf[2*idx] = signal as i16;
                }
            },
            Waveform::Square => {
                for idx in 0..buf.len()/2 {
                    let mut signal = 0.;
                    for i in 0..leftsiglen {
                        if sig[i].phase < sig[i].dutycyclephase {
                            signal += sig[i].amplitude;
                        } else {
                            signal -= sig[i].amplitude;
                        }
                        sig[i].phase += phasestepvec[i];
                        if sig[i].phase <= -2.*::std::f64::consts::PI {
                           sig[i].phase += 2.*::std::f64::consts::PI;
                        } else if sig[i].phase >= 2.*::std::f64::consts::PI {
                           sig[i].phase -= 2.*::std::f64::consts::PI;
                        }
                    }
                    buf[2*idx] = signal as i16;
                }
            },
            Waveform::Triangle => {
                for idx in 0..buf.len()/2 {
                    let mut signal = 0.;
                    for i in 0..leftsiglen {
                        if sig[i].phase < sig[i].dutycyclephase {
                            signal += sig[i].amplitude * (-1. + 2.*sig[i].phase / sig[i].dutycyclephase);
                        } else {
                            signal -= sig[i].amplitude * (-1. + 2.*(sig[i].phase-sig[i].dutycyclephase) / (2.*::std::f64::consts::PI-sig[i].dutycyclephase));
                        }
                        sig[i].phase += phasestepvec[i];
                        if sig[i].phase <= -2.*::std::f64::consts::PI {
                           sig[i].phase += 2.*::std::f64::consts::PI;
                        } else if sig[i].phase >= 2.*::std::f64::consts::PI {
                           sig[i].phase -= 2.*::std::f64::consts::PI;
                        }
                    }
                    buf[2*idx] = signal as i16;
                }
            },
            Waveform::Nowave => {
                for idx in 0..buf.len()/2 {
                    buf[2*idx] = 0;
                }
            },
        }
        match righttype {
            Waveform::Sine => {
                for idx in 0..buf.len()/2 {
                    let mut signal = 0.;
                    for i in leftsiglen..sig.len() {
                        signal += phvec[i].re * sig[i].amplitude;
                        phvec[i] *= ph_diffvec[i];
                    }
                    buf[2*idx+1] = signal as i16;
                }
            },
            Waveform::Square => {
                for idx in 0..buf.len()/2 {
                    let mut signal = 0.;
                    for i in leftsiglen..sig.len() {
                        if sig[i].phase < sig[i].dutycyclephase {
                            signal += sig[i].amplitude;
                        } else {
                            signal -= sig[i].amplitude;
                        }
                        sig[i].phase += phasestepvec[i];
                        if sig[i].phase <= -2.*::std::f64::consts::PI {
                           sig[i].phase += 2.*::std::f64::consts::PI;
                        } else if sig[i].phase >= 2.*::std::f64::consts::PI {
                           sig[i].phase -= 2.*::std::f64::consts::PI;
                        }
                    }
                    buf[2*idx+1] = signal as i16;
                }
            },
            Waveform::Triangle => {
                for idx in 0..buf.len()/2 {
                    let mut signal = 0.;
                    for i in leftsiglen..sig.len() {
                        if sig[i].phase < sig[i].dutycyclephase {
                            signal += sig[i].amplitude * (-1. + 2.*sig[i].phase / sig[i].dutycyclephase);
                        } else {
                            signal -= sig[i].amplitude * (-1. + 2.*(sig[i].phase-sig[i].dutycyclephase) / (2.*::std::f64::consts::PI-sig[i].dutycyclephase));
                        }
                        sig[i].phase += phasestepvec[i];
                        if sig[i].phase <= -2.*::std::f64::consts::PI {
                           sig[i].phase += 2.*::std::f64::consts::PI;
                        } else if sig[i].phase >= 2.*::std::f64::consts::PI {
                           sig[i].phase -= 2.*::std::f64::consts::PI;
                        }
                    }
                    buf[2*idx+1] = signal as i16;
                }
            },
            Waveform::Nowave => {
                for idx in 0..buf.len()/2 {
                    buf[2*idx+1] = 0;
                }
            },
        }
        assert_eq!(io.writei(&buf[..]).unwrap(), buf.len()/2);
    }

    //if pcm.state() != State::Running { pcm.start().unwrap() }; // In case the buffer was larger than 2 seconds, start the stream manually.
    //pcm.drain().unwrap(); // Wait for the stream to finish playback.
}
