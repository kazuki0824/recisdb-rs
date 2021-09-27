#include <stdio.h>
#include <stdlib.h>

#include "pipe_ecm.h"
#include "decoder.h"

decoder *
b25_startup(const decoder_options opt, int no_card)
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

    dec->bcas = create_b_cas_card();
    switch (no_card) {
        case 0:
        {
            if(!dec->bcas) {
                err = "create_b_cas_card failed";
                goto error;
            }
            code = dec->bcas->init(dec->bcas);
            if (code == -3 || code == -4)
                no_card = 1; //fallthrough
            else if(code < 0) {
                err = "bcas->init failed";
                goto error;
            }
            else
                break;
        }
        default:
        {
            *(dec->bcas) = card_default;
            code = dec->bcas->init(dec->bcas);
            if(code < 0) {
                err = "bcas->init failed";
                goto error;
            }
        }
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
