//
// Created by maleicacid on 2021/09/27.
//

#include "IBonDriver.hpp"

extern "C" {
    IBonDriver2* interface_check_2(IBonDriver * i)
    {
        return dynamic_cast<IBonDriver2*>(i);
    }
    IBonDriver3* interface_check_3(IBonDriver2 * i)
    {
        return dynamic_cast<IBonDriver3*>(i);
    }
    const IBonDriver2* interface_check_2_const(const IBonDriver * i)
    {
        return dynamic_cast<const IBonDriver2*>(i);
    }
    const IBonDriver3* interface_check_3_const(const IBonDriver2 * i)
    {
        return dynamic_cast<const IBonDriver3*>(i);
    }
    const BOOL OpenTuner(IBonDriver * b)
    {
        return b->OpenTuner();
    }
	void CloseTuner(IBonDriver * b)
    {
        b->CloseTuner();
    }

	const BOOL SetChannel(IBonDriver * b, const BYTE bCh)
    {
        return b->SetChannel(bCh);
    }
	const float GetSignalLevel(IBonDriver * b)
    {
        return b->GetSignalLevel();
    }

	const DWORD WaitTsStream(IBonDriver * b, const DWORD dwTimeOut = 0)
    {
        return b->WaitTsStream(dwTimeOut);
    }
	const DWORD GetReadyCount();

	const BOOL GetTsStream(IBonDriver * b, BYTE *pDst, DWORD *pdwSize, DWORD *pdwRemain)
    {
        return b->GetTsStream(pDst, pdwSize, pdwRemain);
    }
	const BOOL GetTsStream(IBonDriver * b, BYTE **ppDst, DWORD *pdwSize, DWORD *pdwRemain)
    {
        return b->GetTsStream(ppDst, pdwSize, pdwRemain);
    }

	void PurgeTsStream(IBonDriver * b)
    {
        b->PurgeTsStream();
    }

	void Release(IBonDriver * b)
    {
        b->Release();
    }
}

