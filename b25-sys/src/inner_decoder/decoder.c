#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "pipe_ecm.h"
#include "decoder.h"

decoder *
b25_startup_common(decoder_options opt)
{
    decoder *dec = calloc(1, sizeof(decoder));
    int code;
    const char *err = NULL;

    dec->b25 = create_arib_std_b25();
    if(!dec->b25) {
        err = "create_arib_std_b25 failed";
        goto error;
    }

    code = dec->b25->set_multi2_round(dec->b25, opt.round);
    if(code < 0) {
        err = "set_multi2_round failed";
        goto error;
    }

    code = dec->b25->set_strip(dec->b25, opt.strip);
    if(code < 0) {
        err = "set_strip failed";
        goto error;
    }

    code = dec->b25->set_emm_proc(dec->b25, opt.emm);
    if(code < 0) {
        err = "set_emm_proc failed";
        goto error;
    }

    return dec;

error:
    fprintf(stderr, "%s\n", err);
    free(dec);
    return NULL;
}

decoder *
b25_startup(decoder_options opt)
{
    decoder * dec = b25_startup_common(opt);
    int code;
    const char *err = NULL;

    dec->bcas = create_b_cas_card();
    if(!dec->bcas) {
        err = "create_b_cas_card failed";
        goto error;
    }

    code = dec->bcas->init(dec->bcas);
    if(code < 0) {
        err = "bcas->init failed";
        goto error;
    }

    code = dec->b25->set_b_cas_card(dec->b25, dec->bcas);
    if(code < 0) {
        err = "set_b_cas_card failed";
        goto error;
    }

    return dec;

error:
    fprintf(stderr, "%s\n", err);
    free(dec);
    return NULL;
}

int
b25_shutdown(decoder *dec)
{
    dec->b25->release(dec->b25);
    dec->bcas->release(dec->bcas);
    free(dec);

    return 0;
}

int
b25_decode(decoder *dec, ARIB_STD_B25_BUFFER *sbuf, ARIB_STD_B25_BUFFER *dbuf)
{
    int code;

    code = dec->b25->put(dec->b25, sbuf);
    if(code < 0) {
        fprintf(stderr, "b25->put failed(%d)\n", code);
        return code;
    }

    code = dec->b25->get(dec->b25, dbuf);
    if(code < 0) {
        fprintf(stderr, "b25->get failed(%d)\n", code);
        return code;
    }

    return code;
}

int
b25_finish(decoder *dec, ARIB_STD_B25_BUFFER *sbuf, ARIB_STD_B25_BUFFER *dbuf)
{
    int code;

    code = dec->b25->flush(dec->b25);
    if(code < 0) {
        fprintf(stderr, "b25->flush failed\n");
        return code;
    }

    code = dec->b25->get(dec->b25, dbuf);
    if(code < 0) {
        fprintf(stderr, "b25->get failed\n");
        return code;
    }

    return code;
}

ARIB_STD_B25_BUFFER process_data(decoder* dec, ARIB_STD_B25_BUFFER sbuf)
{
    ARIB_STD_B25_BUFFER dbuf, buf;
    int code = b25_decode(dec, &sbuf, &dbuf);
    if(code < 0) {
        fprintf(stderr, "b25_decode failed (code=%d). fall back to encrypted recording.\n", code);
        buf = sbuf;
    }
    else
    {
        //free(sbuf.data);
        buf = dbuf;
    }
    return buf;
}

#include <winscard.h>
#define B_CAS_BUFFER_MAX (4*1024)

typedef struct {

    SCARDCONTEXT       mng;
    SCARDHANDLE        card;

    uint8_t           *pool;
    LPTSTR             reader;

    uint8_t           *sbuf;
    uint8_t           *rbuf;

    B_CAS_INIT_STATUS  stat;

    B_CAS_ID           id;
    int32_t            id_max;

    B_CAS_PWR_ON_CTRL_INFO pwc;
    int32_t            pwc_max;

} B_CAS_CARD_DEFAULT_DATA;
#define V_CARDS_MAX 10
B_CAS_CARD * init_cas_data_on_heap(int proc_ecm, B_CAS_ID bCasId)
{
    B_CAS_CARD * bcas;
    if (proc_ecm)
    {
        //create instance
        bcas = calloc(1, sizeof(B_CAS_CARD));
        if (bcas == NULL)
            return NULL;

        //override all the funcs
        memcpy(bcas, &card_default, sizeof(B_CAS_CARD));

        //create default data
        B_CAS_CARD_DEFAULT_DATA * data;
        data = calloc(1, sizeof(B_CAS_CARD_DEFAULT_DATA));
        if (data == NULL)
            return NULL;
        data->mng = 0;
        data->card = 0;
        data->pool = NULL;
        data->reader = "local";
        data->sbuf = malloc(B_CAS_BUFFER_MAX);
        data->rbuf = malloc(B_CAS_BUFFER_MAX);
        memcpy(&data->stat, &preset, sizeof(B_CAS_INIT_STATUS));
        data->pwc.data = pwc_data;
        data->pwc.count = 1;
        data->pwc_max = 16;

        data->id_max = V_CARDS_MAX;
        data->id.count = bCasId.count;
        memcpy(data->id.data, bCasId.data, 8 * bCasId.count);

        bcas->private_data = data;
    }
    else
    {
        //create_b_cas_card() and override emm processing
        bcas = create_b_cas_card();
        if (bcas == NULL)
            return NULL;
        //override func
        bcas->proc_emm = card_default.proc_emm;
    }

    return bcas;
}

decoder *
b25_startup_with_debug(decoder_options opt, int proc_ecm, B_CAS_ID bCasId)
{
    int code;
    const char *err = NULL;
    if (!proc_ecm && bCasId.count <=0)
    {
        err = "Debug mode was chosen but no keys are set and proc_ecm is false.";
        goto error;
    }

    decoder * dec = b25_startup_common(opt);

    dec->bcas = init_cas_data_on_heap(proc_ecm, bCasId);
    if(dec->bcas ==NULL) {
        err = "init_cas_data_on_heap failed";
        goto error;
    }

    code = dec->bcas->init(dec->bcas);
    if(code < 0) {
        err = "bcas->init failed";
        goto error;
    }

    code = dec->b25->set_b_cas_card(dec->b25, dec->bcas);
    if(code < 0) {
        err = "set_b_cas_card failed";
        goto error;
    }

    return dec;

    error:
    fprintf(stderr, "%s\n", err);
    free(dec);
    return NULL;

}
