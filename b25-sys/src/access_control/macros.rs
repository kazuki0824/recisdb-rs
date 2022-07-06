#[macro_export]
macro_rules! impl_block_encdec {
    (
        <$($N:ident$(:$b0:ident$(+$b:ident)*)?),*>
        $cipher:ident, $block_size:ty, $par_blocks:ty, $state:ident, $block:ident,
        encrypt: $enc_block:block
        decrypt: $dec_block:block
    ) => {
        impl<$($N$(:$b0$(+$b)*)?),*> cipher::BlockSizeUser for $cipher<$($N),*> {
            type BlockSize = $block_size;
        }

        impl<$($N$(:$b0$(+$b)*)?),*> cipher::BlockEncrypt for $cipher<$($N),*> {
            fn encrypt_with_backend(&self, f: impl cipher::BlockClosure<BlockSize = $block_size>) {
                struct EncBack<'a, $($N$(:$b0$(+$b)*)?),* >(&'a $cipher<$($N),*>);

                impl<'a, $($N$(:$b0$(+$b)*)?),* > cipher::BlockSizeUser for EncBack<'a, $($N),*> {
                    type BlockSize = $block_size;
                }

                impl<'a, $($N$(:$b0$(+$b)*)?),* > cipher::ParBlocksSizeUser for EncBack<'a, $($N),*> {
                    type ParBlocksSize = $par_blocks;
                }

                impl<'a, $($N$(:$b0$(+$b)*)?),* > cipher::BlockBackend for EncBack<'a, $($N),*> {
                    #[inline(always)]
                    fn proc_block(
                        &mut self,
                        mut $block: cipher::inout::InOut<'_, '_, cipher::Block<Self>>
                    ) {
                        let $state: &$cipher<$($N),*> = self.0;
                        $enc_block
                    }
                }

                f.call(&mut EncBack(self))
            }
        }

        impl<$($N$(:$b0$(+$b)*)?),*> cipher::BlockDecrypt for $cipher<$($N),*> {
            fn decrypt_with_backend(&self, f: impl cipher::BlockClosure<BlockSize = $block_size>) {
                struct DecBack<'a, $($N$(:$b0$(+$b)*)?),* >(&'a $cipher<$($N),*>);

                impl<'a, $($N$(:$b0$(+$b)*)?),* > cipher::BlockSizeUser for DecBack<'a, $($N),*> {
                    type BlockSize = $block_size;
                }

                impl<'a, $($N$(:$b0$(+$b)*)?),* > cipher::ParBlocksSizeUser for DecBack<'a, $($N),*> {
                    type ParBlocksSize = $par_blocks;
                }

                impl<'a, $($N$(:$b0$(+$b)*)?),* > cipher::BlockBackend for DecBack<'a, $($N),*> {
                    #[inline(always)]
                    fn proc_block(
                        &mut self,
                        mut $block: cipher::inout::InOut<'_, '_, cipher::Block<Self>>
                    ) {
                        let $state: &$cipher<$($N),*> = self.0;
                        $dec_block
                    }
                }

                f.call(&mut DecBack(self))
            }
        }
    };
    (
        $cipher:ident, $block_size:ty, $par_blocks:ty, $state:ident, $block:ident,
        encrypt: $enc_block:block
        decrypt: $dec_block:block
    ) => {
        $crate::impl_block_encdec!(
            <> $cipher, $block_size, $par_blocks, $state, $block,
            encrypt: $enc_block
            decrypt: $dec_block
        );
    };
}
