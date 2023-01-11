use comfy_table::Table;
use sysinfo::{CpuExt, System, SystemExt};
use Gui_src::{Colour, Information2, View};
pub trait Checking {
    fn checking_cpu(n: i32) -> Table {
        let mut sys = System::new();
        let mut t = 0;
        let mut m = vec![];
        'life: loop {
            t += 1;
            sys.refresh_cpu();
            let mut r = 0;
            sys.cpus().iter().for_each(|cpu| {
                r += 1;
                m.push(vec![
                    format!("CPU{r}"),
                    format!("To[{t}]Refresh"),
                    format!("{}", cpu.cpu_usage()),
                ]);
            });
            std::thread::sleep(std::time::Duration::from_millis(1000));
            if t > n {
                break 'life;
            }
        }
        Colour::Monitoring.table2(Information2 {
            list: vec!["Name", "Frequency", "Employ"],
            data: m,
        })
    }
}
