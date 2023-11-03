
#include "../IBonDriver.hpp"


const BOOL CPP_GetTsStream(IBonDriver * b, BYTE *pDst, DWORD *pdwSize, DWORD *pdwRemain)
{
    return b->GetTsStream(pDst, pdwSize, pdwRemain);
}
const BOOL CPP_GetTsStream2(IBonDriver * b, BYTE **ppDst, DWORD *pdwSize, DWORD *pdwRemain)
{
    return b->GetTsStream(ppDst, pdwSize, pdwRemain);
}

extern "C" {
    const BOOL C_OpenTuner(IBonDriver * b)
    {
        return b->OpenTuner();
    }
    void C_CloseTuner(IBonDriver * b)
    {
        b->CloseTuner();
    }

    const BOOL C_SetChannel(IBonDriver * b, const BYTE bCh)
    {
        return b->SetChannel(bCh);
    }
    const float C_GetSignalLevel(IBonDriver * b)
    {
        return b->GetSignalLevel();
    }

    const DWORD C_WaitTsStream(IBonDriver * b, const DWORD dwTimeOut = 0)
    {
        return b->WaitTsStream(dwTimeOut);
    }
    const DWORD C_GetReadyCount();

    const BOOL C_GetTsStream(IBonDriver * b, BYTE *pDst, DWORD *pdwSize, DWORD *pdwRemain)
    {
        return CPP_GetTsStream(b, pDst, pdwSize, pdwRemain);
    }
    const BOOL C_GetTsStream2(IBonDriver * b, BYTE **ppDst, DWORD *pdwSize, DWORD *pdwRemain)
    {
        return CPP_GetTsStream2(b, ppDst,pdwSize, pdwRemain);
    }

    void C_PurgeTsStream(IBonDriver * b)
    {
        b->PurgeTsStream();
    }

    void C_Release(IBonDriver * b)
    {
        b->Release();
    }
}
