/*
Copyright (c) 2015, Daniel Renninghoff
All rights reserved.

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software without
   specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

extern crate gtk;
extern crate glib;
extern crate yahtzeesolve;

use std::cell::RefCell;
use gtk::traits::*;
use gtk::signal::Inhibit;
use std::sync::mpsc;
use std::thread;
use yahtzeesolve::LookupTable;

thread_local!(
    static GLOBAL: RefCell<Option<(gtk::ProgressBar, f64)>> = RefCell::new(None)
);

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let glade_src = include_str!("gui.glade");
    let builder = gtk::widgets::Builder::new_from_string(glade_src).unwrap();
    let window: gtk::Window = builder.get_object("applicationwindow1").unwrap();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(true)
    });
    window.show_all();

    let lookup = LookupTable::from_file("probs.dat").unwrap_or_else(|_| {
        let dialog: gtk::Dialog = builder.get_object("dialog1").unwrap();
        let progressbar: gtk::ProgressBar = builder.get_object("progressbar1").unwrap();
        GLOBAL.with(move |global| {
            *global.borrow_mut() = Some((progressbar, 0.0))
        });

        let t = thread::spawn(move || {
            let (tx, rx) = mpsc::channel();
            let t2 = LookupTable::generate(tx);
            for _ in 0..100 {
                rx.recv().unwrap();
                println!("test");
                glib::idle_add(|| {
                    GLOBAL.with(|global| {
                        if let Some((ref pb, mut progr)) = *global.borrow() {
                            println!("test2");
                            progr += 0.01;
                            pb.set_fraction(progr);
                        }
                    });
                    glib::Continue(false)
                });
            }
            t2.join().unwrap()
        });

        gtk::Dialog::run(&dialog);
        let lookup2 = t.join().unwrap();
        lookup2.write_to_file("probs.dat").unwrap();
        lookup2
    });

    gtk::main();
}
