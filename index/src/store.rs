use exocore_data;

// TODO: Needs to decrypt
/* TODO: Use thread local for decryption + decompression.
         Impose a limit on decrypt and decompress block size...
         https://doc.rust-lang.org/std/macro.thread_local.html
 */

struct Store<T: exocore_data::transport::Transport, P>
where
    P: for<'pers> exocore_data::chain::Persistence<'pers>,
{
    data_engine: exocore_data::Engine<T, P>,
}

impl<T: exocore_data::transport::Transport, P> Store<T, P>
where
    P: for<'pers> exocore_data::chain::Persistence<'pers>,
{
    pub fn new() {}
}
