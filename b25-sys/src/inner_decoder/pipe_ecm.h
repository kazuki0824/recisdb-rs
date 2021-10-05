//
// Created by kazuki on 2020/11/19.
//

#ifndef RECISDB_RUST_PIPE_ECM_H
#define RECISDB_RUST_PIPE_ECM_H

#define ARIB25_API_STATIC_DEFINE
#include "arib25/b_cas_card.h"

extern B_CAS_CARD card_default;
B_CAS_INIT_STATUS preset;
B_CAS_PWR_ON_CTRL pwc_data[1];

/* cbindgen 0.20.0 */
#include <stdint.h>

void post_scramble_key(const uint8_t *src, uintptr_t len, uint8_t *dst);

void post_emm(const uint8_t *src, uintptr_t len);


#endif //RECISDB_RUST_PIPE_ECM_H
