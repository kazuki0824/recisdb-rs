//
// Created by kazuki on 2020/11/19.
//

#include "pipe_ecm.h"
#include "b_cas_card_error_code.h"

#include <stdio.h>
#include <sys/types.h>
#if _MSC_VER > 1920
#define _MSVC_C
#include <Windows.h>
static HANDLE rmq, smq;
#else
#include <unistd.h>
#include <mqueue.h>
static mqd_t rmq, smq;
#endif
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

static int64_t id[1] = {0x000000000000};
static B_CAS_PWR_ON_CTRL pwc_data[1] = {0};
static uint8_t sbuf[B_CAS_BUFFER_MAX]={0};
static uint8_t rbuf[B_CAS_BUFFER_MAX]={0};
static B_CAS_CARD_DEFAULT_DATA bcasprv = {
        0,
        0,
        NULL,
        "local",
        sbuf,
        rbuf,
        {
                {0x36,0x31,0x04,0x66,0x4b,0x17,0xea,0x5c, 0x32,0xdf,0x9c,0xf5,0xc4,0xc3,0x6c,0x1b,
                 0xec,0x99,0x39,0x21,0x68,0x9d,0x4b,0xb7, 0xb7,0x4e,0x40,0x84,0x0d,0x2e,0x7d,0x98},
                {0xfe,0x27,0x19,0x99,0x19,0x69,0x09,0x11},
                0xfe27199919690911,
                0x0000,
                0x0005
        },
        {
            id,
            1
        },
        16,
        {
            pwc_data,
            1
        },
        16,
};

#ifdef _MSVC_C
static inline void send_ecm(unsigned char * src, int sz, unsigned char ks[16])
{
    DWORD cbWritten, cbRead = 0;
    BOOL fResult = WriteFile(smq, 
     src, 
     (DWORD) sz,  
     &cbWritten, 
     (LPOVERLAPPED) NULL); 
    if (!fResult) 
    { 
        printf("WriteFile failed with %d.\n", GetLastError()); 
        return; 
    }

    fResult = ReadFile(rmq, 
        ks, 
        16, 
        &cbRead, 
        (LPOVERLAPPED) NULL); 
    if (!fResult) 
    { 
        printf("ReadFile failed with %d.\n", GetLastError()); 
        return; 
    } 
}
static void deinit(void *bcas)
{
    return;
}
static int init(void *bcas)
{
    DWORD pid = GetCurrentProcessId();
    //create mailslot
    LPTSTR c[18], p[18];
    sprintf(c, "/tmp_%d_mqecm_c", pid);
    sprintf(p, "/tmp_%d_mqecm_p", pid);

    smq = CreateMailslot(
        (LPCTSTR)c, 
        10, 
        2000, 
        (LPSECURITY_ATTRIBUTES) NULL 
    );

    rmq = CreateMailslot(
        (LPCTSTR)c, 
        16, 
        2000, 
        (LPSECURITY_ATTRIBUTES) NULL 
    );

    if ((smq == INVALID_HANDLE_VALUE) || (rmq == INVALID_HANDLE_VALUE))
    {
        return B_CAS_CARD_ERROR_NOT_INITIALIZED;
    }

    return 0;
}
#else
static inline void send_ecm(unsigned char * src, int sz, unsigned char ks[16])
{
    //send ECM
    mq_send(smq, (const char *)src, sz, 10);
    //recv a scramble key
    unsigned int prio = 0;
    mq_receive(rmq, (char *)ks, 16, &prio);
}

static void deinit(void *bcas)
{
    //release pipe
    mq_close(rmq);
    mq_close(smq);
    return;
}
#define MQ_FILEMODE (S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH)
static int init(void *bcas)
{
    //getpid()
    pid_t pid = getpid();
    //connect to mq
    struct mq_attr attr1;
    attr1.mq_flags = 0;
    attr1.mq_maxmsg = 10;
    attr1.mq_msgsize = 30;
    attr1.mq_curmsgs = 0;
    struct mq_attr attr2;
    attr2.mq_flags = 0;
    attr2.mq_maxmsg = 10;
    attr2.mq_msgsize = 16;
    attr2.mq_curmsgs = 0;

    char c[18], p[18];
    sprintf(c, "/tmp_%d_mqecm_c", pid);
    sprintf(p, "/tmp_%d_mqecm_p", pid);
    smq = mq_open(c, O_WRONLY | O_CREAT, MQ_FILEMODE, &attr1);
    rmq = mq_open(p, O_RDONLY | O_CREAT, MQ_FILEMODE, &attr2);
    //if not, return -2 = B_CAS_CARD_ERROR_NOT_INITIALIZED
    if (rmq <0 || smq <0)
        return B_CAS_CARD_ERROR_NOT_INITIALIZED;

    return 0;
}
#endif

static int get_init_status(void *bcas, B_CAS_INIT_STATUS *stat)
{
    *stat = bcasprv.stat;
    return 0;
}
static int get_id(void *bcas, B_CAS_ID *dst)
{
    *dst = bcasprv.id;
    return 0;
}
static int get_pwr_on_ctrl(void *bcas, B_CAS_PWR_ON_CTRL_INFO *dst)
{
    *dst = bcasprv.pwc;
    return 0;
}

#include <stdio.h>
static int proc_ecm(void *bcas, B_CAS_ECM_RESULT *dst, uint8_t *src, int len)
{
    post_scramble_key(src, len, dst->scramble_key);
    dst->return_code = 0x0800;
    return 0;
}
static int proc_emm(void *bcas, uint8_t *src, int len)
{
    post_emm(src, len);
    return 0;
}

B_CAS_CARD card_default = {&bcasprv, deinit, init, get_init_status, get_id, get_pwr_on_ctrl, proc_ecm, proc_emm};