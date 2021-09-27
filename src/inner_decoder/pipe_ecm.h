//
// Created by kazuki on 2020/11/19.
//

#ifndef RECISDB_RUST_PIPE_ECM_H
#define RECISDB_RUST_PIPE_ECM_H

#include "arib25/b_cas_card.h"
B_CAS_CARD card_default;

/* cbindgen 0.20.0 */
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

void post_scramble_key(const uint8_t *src, uintptr_t len, uint8_t *dst);

void post_emm(const uint8_t *src, uintptr_t len);



#endif //RECISDB_RUST_PIPE_ECM_H
