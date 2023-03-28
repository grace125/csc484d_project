use bevy::prelude::*;
use knyst::{prelude::*, wavetable::WavetableOscillatorOwned};
use project::{KnystAudioPlugin, AudioCommands, AudioStartupSet};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(KnystAudioPlugin)
        .add_startup_system(initialize_graph.after(AudioStartupSet))
        .run();
}

fn initialize_graph(mut a_commands: ResMut<AudioCommands>) {
    let node0 = a_commands.push(
        WavetableOscillatorOwned::new(Wavetable::sine()),
        inputs!(("freq" : 440.)),
    );
    let modulator = a_commands.push(
        WavetableOscillatorOwned::new(Wavetable::sine()),
        inputs!(("freq" : 5.)),
    );
    let mod_amp = a_commands.push(Mult, inputs!((0 ; modulator.out(0)), (1 : 0.25)));
    let amp = a_commands.push(
        Mult,
        inputs!((0 ; node0.out(0)), (1 : 0.5 ; mod_amp.out(0))),
    );
    a_commands.connect(amp.to_graph_out());
    a_commands.connect(amp.to_graph_out().to_index(1));
}