use exocore_data;

// TODO: Needs to decrypt

struct Store<T: exocore_data::transport::Transport, P: exocore_data::chain::ChainPersistence> {
    data_engine: exocore_data::Engine<T, P>,
}

impl<T: exocore_data::transport::Transport, P: exocore_data::chain::ChainPersistence> Store<T, P> {
    pub fn new() {}
}
