// Copyright 2017  Jos van den Oever <jos@vandenoever.info>
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation; either version 2 of
// the License or (at your option) version 3 or any later version
// accepted by the membership of KDE e.V. (or its successor approved
// by the membership of KDE e.V.), which shall act as a proxy
// defined in Section 14 of version 3 of the license.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use interface::*;
use sysinfo::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use libc::pid_t;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver, RecvTimeoutError};

struct ProcessItem {
    row: usize,
    tasks: Vec<pid_t>,
    process: Process,
}

#[derive(Default)]
struct ProcessTree {
    top: Vec<pid_t>,
    processes: HashMap<pid_t, ProcessItem>,
    cpusum: f32,
}

enum ChangeState {
    Active,
    Inactive,
    Quit
}

pub struct Processes {
    emit: ProcessesEmitter,
    model: ProcessesTree,
    p: ProcessTree,
    incoming: Arc<Mutex<Option<ProcessTree>>>,
    active: bool,
    channel: Sender<ChangeState>
}

fn check_process_hierarchy(parent: Option<pid_t>, processes: &HashMap<pid_t, Process>) {
    for (pid, process) in processes {
        assert_eq!(process.pid, *pid);
        if !parent.is_none() {
            assert_eq!(process.parent, parent);
        }
        check_process_hierarchy(Some(*pid), &process.tasks);
    }
}

fn collect_processes(
    tasks: &HashMap<pid_t, Process>,
    mut processes: &mut HashMap<pid_t, ProcessItem>,
) -> f32 {
    let mut cpusum = 0.0;
    for process in tasks.values() {
        processes.insert(
            process.pid,
            ProcessItem {
                row: 0,
                tasks: Vec::new(),
                process: process.clone(),
            },
        );
        let s = collect_processes(&process.tasks, &mut processes);
        cpusum += process.cpu_usage + s;
    }
    cpusum
}

// reconstruct process hierarchy
fn handle_tasks(processes: &mut HashMap<pid_t, ProcessItem>) -> Vec<pid_t> {
    let mut top = Vec::new();
    let pids: Vec<pid_t> = processes.keys().cloned().collect();
    for pid in pids {
        if let Some(parent) = processes[&pid].process.parent {
            let p = processes.get_mut(&parent).unwrap();
            p.tasks.push(pid);
        } else {
            top.push(pid);
        }
    }
    top
}

fn update_rows(list: &[pid_t], processes: &mut HashMap<pid_t, ProcessItem>) {
    for (row, pid) in list.iter().enumerate() {
        processes.get_mut(pid).unwrap().row = row;
        let l = processes[pid].tasks.clone();
        update_rows(&l, processes);
    }
}

fn sort_tasks(p: &mut ProcessTree) {
    for process in p.processes.values_mut() {
        process.tasks.sort();
    }
    p.top.sort();
    update_rows(&p.top, &mut p.processes);
}

fn update() -> ProcessTree {
    let mut p = ProcessTree::default();
    let mut sysinfo = System::new();
    sysinfo.refresh_processes();
    let list = sysinfo.get_process_list();
    check_process_hierarchy(None, list);
    p.cpusum = collect_processes(list, &mut p.processes);
    p.top = handle_tasks(&mut p.processes);
    sort_tasks(&mut p);
    p
}

fn update_thread(
    emit: ProcessesEmitter,
    incoming: Arc<Mutex<Option<ProcessTree>>>,
    mut active: bool,
    status_channel: Receiver<ChangeState>,
) {
    thread::spawn(move || {
        loop {
            let timeout = if active {
                *incoming.lock().unwrap() = Some(update());
                emit.new_data_ready(None);
                Duration::from_secs(1)
            } else {
                Duration::from_secs(10_000)
            };
            match status_channel.recv_timeout(timeout) {
                Err(RecvTimeoutError::Timeout) => {},
                Err(RecvTimeoutError::Disconnected)
                | Ok(ChangeState::Quit) => { return; },
                Ok(ChangeState::Active) => { active = true; },
                Ok(ChangeState::Inactive) => { active = false; },
            }
        }
    });
}

impl Processes {
    fn get(&self, item: usize) -> &ProcessItem {
        let pid = item as pid_t;
        &self.p.processes[&pid]
    }
    fn process(&self, item: usize) -> &Process {
        let pid = item as pid_t;
        &self.p.processes[&pid].process
    }
}

fn move_process(
    pid: pid_t,
    amap: &mut HashMap<pid_t, ProcessItem>,
    bmap: &mut HashMap<pid_t, ProcessItem>,
) {
    if let Some(e) = bmap.remove(&pid) {
        amap.insert(pid, e);
        let ts = amap[&pid].tasks.clone();
        for t in ts {
            move_process(t, amap, bmap);
        }
    }
}

fn remove_row(
    model: &ProcessesTree,
    parent: pid_t,
    row: usize,
    map: &mut HashMap<pid_t, ProcessItem>,
) {
    let pid = map[&parent].tasks[row];
    println!(
        "removing {} '{}' {}",
        pid,
        map[&pid].process.exe,
        map[&pid].process.cmd.join(" ")
    );
    model.begin_remove_rows(Some(parent as usize), row, row);
    map.remove(&pid);
    let len = {
        let tasks = &mut map.get_mut(&parent).unwrap().tasks;
        tasks.remove(row);
        tasks.len()
    };
    for r in row..len {
        let pid = map.get_mut(&parent).unwrap().tasks[r];
        map.get_mut(&pid).unwrap().row = r;
    }
    model.end_remove_rows();
}

fn insert_row(
    model: &ProcessesTree,
    parent: pid_t,
    row: usize,
    map: &mut HashMap<pid_t, ProcessItem>,
    pid: pid_t,
    source: &mut HashMap<pid_t, ProcessItem>,
) {
    println!(
        "adding {} '{}' {}",
        pid,
        source[&pid].process.exe,
        source[&pid].process.cmd.join(" ")
    );
    model.begin_insert_rows(Some(parent as usize), row, row);
    move_process(pid, map, source);
    let len = {
        let tasks = &mut map.get_mut(&parent).unwrap().tasks;
        tasks.insert(row, pid);
        tasks.len()
    };
    for r in row..len {
        let pid = map.get_mut(&parent).unwrap().tasks[r];
        map.get_mut(&pid).unwrap().row = r;
    }
    model.end_insert_rows();
}

fn cmp_f32(a: f32, b: f32) -> bool {
    ((a - b) / a).abs() < 0.01
}

fn sync_row(model: &ProcessesTree, pid: pid_t, a: &mut Process, b: &Process) -> f32 {
    let mut changed = a.name != b.name;
    if changed {
        a.name.clone_from(&b.name);
    }
    if !cmp_f32(a.cpu_usage, b.cpu_usage) {
        a.cpu_usage = b.cpu_usage;
        changed = true;
    }
    if a.cmd != b.cmd {
        a.cmd.clone_from(&b.cmd);
        changed = true;
    }
    if a.exe != b.exe {
        a.exe.clone_from(&b.exe);
        changed = true;
    }
    if a.memory != b.memory {
        a.memory = b.memory;
        changed = true;
    }
    if changed {
        model.data_changed(pid as usize, pid as usize);
    }
    b.cpu_usage
}

fn sync_tree(
    model: &ProcessesTree,
    parent: pid_t,
    amap: &mut HashMap<pid_t, ProcessItem>,
    bmap: &mut HashMap<pid_t, ProcessItem>,
) -> f32 {
    let mut a = 0;
    let mut b = 0;
    let mut alen = amap[&parent].tasks.len();
    let blen = bmap[&parent].tasks.len();
    let mut cpu_total = bmap[&parent].process.cpu_usage;

    while a < alen && b < blen {
        let apid = amap[&parent].tasks[a];
        let bpid = bmap[&parent].tasks[b];
        if apid < bpid {
            // a process has disappeared
            remove_row(model, parent, a, amap);
            alen -= 1;
        } else if apid > bpid {
            // a process has appeared
            insert_row(model, parent, a, amap, bpid, bmap);
            cpu_total += amap[&bpid].process.cpu_usage;
            a += 1;
            alen += 1;
            b += 1;
        } else {
            cpu_total += sync_row(
                model,
                apid,
                &mut amap.get_mut(&apid).unwrap().process,
                &bmap[&apid].process,
            );
            cpu_total += sync_tree(model, apid, amap, bmap);
            a += 1;
            b += 1;
        }
    }
    while a < blen {
        let bpid = bmap[&parent].tasks[b];
        insert_row(model, parent, a, amap, bpid, bmap);
        a += 1;
        alen += 1;
        b += 1;
    }
    while b < alen {
        remove_row(model, parent, a, amap);
        alen -= 1;
    }
    if !cmp_f32(cpu_total, bmap[&parent].process.cpu_usage) {
        amap.get_mut(&parent).unwrap().process.cpu_usage = cpu_total;
        model.data_changed(parent as usize, parent as usize);
    }
    assert_eq!(a, b);
    cpu_total
}

impl ProcessesTrait for Processes {
    fn new(emit: ProcessesEmitter, model: ProcessesTree) -> Processes {
        let (tx, rx) = channel();
        let p = Processes {
            emit: emit.clone(),
            model: model,
            p: ProcessTree::default(),
            incoming: Arc::new(Mutex::new(None)),
            active: false,
            channel: tx,
        };
        update_thread(emit, p.incoming.clone(), p.active, rx);
        p
    }
    fn emit(&self) -> &ProcessesEmitter {
        &self.emit
    }
    fn row_count(&self, item: Option<usize>) -> usize {
        if let Some(item) = item {
            self.get(item).tasks.len()
        } else {
            self.p.top.len()
        }
    }
    fn index(&self, item: Option<usize>, row: usize) -> usize {
        if let Some(item) = item {
            self.get(item).tasks[row] as usize
        } else {
            self.p.top[row] as usize
        }
    }
    fn parent(&self, item: usize) -> Option<usize> {
        self.get(item).process.parent.map(|pid| pid as usize)
    }
    fn can_fetch_more(&self, item: Option<usize>) -> bool {
        if item.is_some() || !self.active {
            return false;
        }
        if let Ok(ref incoming) = self.incoming.try_lock() {
            incoming.is_some()
        } else {
            false
        }
    }
    fn fetch_more(&mut self, item: Option<usize>) {
        if item.is_some() || !self.active {
            return;
        }
        let new = if let Ok(ref mut incoming) = self.incoming.try_lock() {
            incoming.take()
        } else {
            None
        };
        if let Some(mut new) = new {
            // alert! at the top level, only adding is supported!
            if self.p.top.is_empty() {
                self.model.begin_reset_model();
                self.p = new;
                self.model.end_reset_model();
            } else {
                let top = self.p.top.clone();
                for pid in top {
                    sync_tree(&self.model, pid, &mut self.p.processes, &mut new.processes);
                }
            }
        }
    }
    fn row(&self, item: usize) -> usize {
        self.get(item).row
    }
    fn pid(&self, item: usize) -> u32 {
        self.process(item).pid as u32
    }
    fn uid(&self, item: usize) -> u32 {
        self.process(item).uid as u32
    }
    fn cpu_usage(&self, item: usize) -> f32 {
        self.process(item).cpu_usage
    }
    fn cpu_percentage(&self, item: usize) -> u8 {
        let cpu = self.process(item).cpu_usage / self.p.cpusum;
        (cpu * 100.0) as u8
    }
    fn memory(&self, item: usize) -> u64 {
        self.process(item).memory
    }
    fn name(&self, item: usize) -> &str {
        &self.process(item).name
    }
    fn cmd(&self, item: usize) -> String {
        self.process(item).cmd.join(" ")
    }
    fn active(&self) -> bool {
        self.active
    }
    fn set_active(&mut self, active: bool) {
        if self.active != active {
            self.active = active;
            if active {
                self.channel.send(ChangeState::Active)
            } else {
                self.channel.send(ChangeState::Inactive)
            }.expect("Process thread died.");
        }
    }
}

impl Drop for Processes {
    fn drop(&mut self) {
        self.channel.send(ChangeState::Quit).expect("Process thread died.");
    }
}
