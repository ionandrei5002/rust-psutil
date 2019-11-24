use psutil::memory::{VirtualMemory, SwapMemory};
use std::thread::sleep;
use std::time::Duration;
use std::ops::Add;
use clap::*;

fn output_per_cpu(per_cpu: &Option<Vec<f64>>) -> String {
    let mut result = String::from("");
    for cpu in (*per_cpu).as_ref().unwrap() {
        result = result.add(format!("{:.1}", cpu).as_str());
        result = result.add("% ");
    }

    return result;
}

fn main() {
    let matches = App::new("rust-psutil")
        .version("0.1.0")
        .author("Sebastian Andrei Ion")
        .about("psutil")
        .arg(Arg::with_name("pcpu").short("p").takes_value(false).help("Per CPU %"))
        .arg(Arg::with_name("acpu").short("a").takes_value(false).help("Avg CPU %"))
        .arg(Arg::with_name("vmem").short("m").takes_value(false).help("Mem Usage"))
        .arg(Arg::with_name("smem").short("w").takes_value(false).help("Swap Mem Usage"))
        .arg(Arg::with_name("io").short("i").takes_value(false).help("IO Usage"))
        .arg(Arg::with_name("net").short("k").takes_value(false).help("Net Usage"))
        .get_matches();

    let mut components  = String::from("");
    let separator   = " ";

    let mut per_cpu: Option<Vec<f64>> = None;
    let mut avg_cpu: Option<f64> = None;
    let mut memory: Option<VirtualMemory> = None;
    let mut swap: Option<SwapMemory> = None;
    let mut io_ul: Option<u64> = None;
    let mut io_dl: Option<u64> = None;
    let mut net_ul: Option<f64> = None;
    let mut net_dl: Option<f64> = None;

    // percentage per cpu
    match matches.occurrences_of("pcpu") {
        1 => {
            components = components.add("p");
            per_cpu = match psutil::cpu::cpu_percent_percpu(1.0) {
                Ok(per_cpu) => {
                    Some(per_cpu)
                },
                Err(_) => None
            };
        },
        0 | _ => {},
    }
    // system wide cpu percentage
    match matches.occurrences_of("acpu") {
        1 => {
            components = components.add("a");
            avg_cpu = match psutil::cpu::cpu_percent(1.0) {
                Ok(avg_cpu) => {
                    Some(avg_cpu)
                },
                Err(_) => None
            };
        },
        0 | _ => {},
    }
    // virtual memory
    match matches.occurrences_of("vmem") {
        1 => {
            components = components.add("m");
            memory = match psutil::memory::virtual_memory() {
                Ok(memory) => Some(memory),
                Err(_) => None
            };
        },
        0 | _ => {},
    }
    // swap memory
    match matches.occurrences_of("smem") {
        1 => {
            components = components.add("w");
            swap = match psutil::memory::swap_memory() {
                Ok(swap) => Some(swap),
                Err(_) => None
            };
        },
        0 | _ => {},
    }
    // io counters
    match matches.occurrences_of("io") {
        1 => {
            components = components.add("i");
            let mut counters = psutil::disk::DiskIOCountersCollector::default();
            let past_counters = counters.disk_io_counters_perdisk(false);
            sleep(Duration::from_secs(1));
            let current_counters = counters.disk_io_counters_perdisk(false);
            if past_counters.is_err() || current_counters.is_err() {
                io_ul = None;
                io_dl = None;
            } else {
                let xfer_start = past_counters.unwrap();
                let xfer_finish = current_counters.unwrap();

                io_ul = Some((xfer_finish[&String::from("sda")].read_bytes - xfer_start[&String::from("sda")].read_bytes) / 1024);
                io_dl = Some((xfer_finish[&String::from("sda")].write_bytes - xfer_start[&String::from("sda")].write_bytes) / 1024);
            }
        },
        0 | _ => {},
    }
    // network counters
    match matches.occurrences_of("net") {
        1 => {
            components = components.add("k");
            let mut counters = psutil::network::NetIOCountersCollector::default();
            let past_counters = counters.net_io_counters(true);
            sleep(Duration::from_secs(1));
            let current_counters = counters.net_io_counters(true);
            if past_counters.is_err() || current_counters.is_err() {
                net_ul = None;
                net_dl = None;
            } else {
                let xfer_start = past_counters.unwrap();
                let xfer_finish = current_counters.unwrap();

                net_ul = Some((xfer_finish.bytes_send - xfer_start.bytes_send) as f64 / 1024 as f64);
                net_dl = Some((xfer_finish.bytes_recv - xfer_start.bytes_recv) as f64 / 1024 as f64);
            }
        },
        0 | _ => {},
    }

    let mut output = String::from("");
    output = output.add(separator);

    for char in components.chars() {
        if char == 'p' && per_cpu != None {
            output = output.add(output_per_cpu(&per_cpu).as_str());
            output = output.add(separator);
        }
        if char == 'a' && avg_cpu != None {
            output = output.add("avgCPU:");
            output = output.add(separator);
            output = output.add(format!("{:.2}", avg_cpu.unwrap()).as_str());
            output = output.add("%");
            output = output.add(separator);
        }
        if char == 'm' {
            let mem = memory.as_ref();
            if mem.is_none() == false {
                output = output.add("MEM:");
                output = output.add(separator);
                {
                    output = output.add(format!("{:.2}", ((mem.unwrap().shared + mem.unwrap().used) as f64) / (1024 * 1024 * 1024) as f64).as_str());
                }
                output = output.add("/");
                {
                    output = output.add(format!("{:.2}", (mem.unwrap().total as f64) / (1024 * 1024 * 1024) as f64).as_str());
                }
                output = output.add(separator);
                output = output.add("GB");
                output = output.add(separator);
            }
        }
        if char == 'w' {
            let mem = swap.as_ref();
            if mem.is_none() == false {
                output = output.add("Swap:");
                output = output.add(separator);
                {
                    output = output.add(format!("{:.2}", (mem.unwrap().used as f64) / (1024 * 1024 * 1024) as f64).as_str());
                }
                output = output.add("/");
                {
                    output = output.add(format!("{:.2}", (mem.unwrap().total as f64) / (1024 * 1024 * 1024) as f64).as_str());
                }
                output = output.add(separator);
                output = output.add("GB");
                output = output.add(separator);
            }
        }
        if char == 'k' && net_ul != None  && net_dl != None {
            output = output.add("NET:");
            output = output.add(separator);
            output = output.add(format!("{:.2}", net_ul.unwrap()).as_str());
            output = output.add(separator);
            output = output.add(format!("{:.2}", net_dl.unwrap()).as_str());
            output = output.add(separator);
            output = output.add("kB/s");
            output = output.add(separator);
        }
        if char == 'i' && io_ul != None  && io_dl != None {
            output = output.add("IO:");
            output = output.add(separator);
            output = output.add(format!("{:.2}", io_ul.unwrap()).as_str());
            output = output.add(separator);
            output = output.add(format!("{:.2}", io_dl.unwrap()).as_str());
            output = output.add(separator);
            output = output.add("kB/s");
            output = output.add(separator);
        }
    }

    println!("{}", output)
}
