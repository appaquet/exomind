use exocore_data;

// TODO: Needs to decrypt
/* TODO: Use thread local for decryption + decompression.
        Impose a limit on decrypt and decompress block size...
        https://doc.rust-lang.org/std/macro.thread_local.html
*/

struct Store<T, CP, PP>
where
    T: exocore_data::transport::Transport,
    CP: exocore_data::chain::Persistence,
    PP: exocore_data::pending::Persistence,
{
    data_engine: exocore_data::Engine<T, CP, PP>,
}

impl<T, CP, PP> Store<T, CP, PP>
where
    T: exocore_data::transport::Transport,
    CP: exocore_data::chain::Persistence,
    PP: exocore_data::pending::Persistence,
{
    pub fn new() {}
}
