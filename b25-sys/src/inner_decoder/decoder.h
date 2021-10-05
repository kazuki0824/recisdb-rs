//
// Created by kazuki on 2020/03/10.
//

#ifndef RECISDB_RUST_DECODER_H
#define RECISDB_RUST_DECODER_H

#define ARIB25_API_STATIC_DEFINE
#include <arib25/arib_std_b25.h>
#include <arib25/b_cas_card.h>


typedef struct {
    ARIB_STD_B25 *b25;
    B_CAS_CARD *bcas;
} decoder;

typedef struct {
    int round;
    int strip;
    int emm;
} decoder_options;

decoder *b25_startup(const decoder_options opt);
int b25_shutdown(decoder *dec);
int b25_decode(decoder *dec,
               ARIB_STD_B25_BUFFER *sbuf,
               ARIB_STD_B25_BUFFER *dbuf);
int b25_finish(decoder *dec,
               ARIB_STD_B25_BUFFER *sbuf,
               ARIB_STD_B25_BUFFER *dbuf);

ARIB_STD_B25_BUFFER process_data(decoder* dec, ARIB_STD_B25_BUFFER sbuf);

decoder *
b25_startup_with_debug(decoder_options opt, int proc_ecm, B_CAS_ID bCasId);

#endif //RECISDB_RUST_DECODER_H
