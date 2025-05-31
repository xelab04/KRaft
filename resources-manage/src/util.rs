pub fn convert_cpu(cpu: String) -> u32 {
    let mut cpu_m:u32;
    if cpu.ends_with("m") {
        cpu_m = cpu.strip_suffix("m").unwrap().parse().unwrap();
    }
    else {
        match cpu.parse::<u32>() {
            Ok(val) => {cpu_m = val;},
            Err(e) => {cpu_m=0;}
        }
    }
    cpu_m
}


pub fn convert_memory(mem: String) -> u32 {
    let mut mem_m: u32;

    if (mem.ends_with("Mi")) {
        mem_m = mem.strip_suffix("Mi").unwrap().parse().unwrap()
    }
    else if (mem.ends_with("Gi")) {
        mem_m = mem.strip_suffix("Gi").unwrap().parse().unwrap();
        mem_m *= 1024
    }
    else {
        mem_m = 0;
    }

    mem_m
}
