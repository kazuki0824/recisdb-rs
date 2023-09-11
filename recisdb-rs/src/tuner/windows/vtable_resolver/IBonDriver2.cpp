#include "../IBonDriver.hpp"

extern "C" {
    const LPCTSTR C_EnumTuningSpace(IBonDriver2 * b, const DWORD dwSpace)
    {
        return b->EnumTuningSpace(dwSpace);
    }
	const LPCTSTR C_EnumChannelName2(IBonDriver2 * b, const DWORD dwSpace, const DWORD dwChannel)
    {
        return b->EnumChannelName(dwSpace, dwChannel);
    }
	const BOOL C_SetChannel2(IBonDriver2 * b, const DWORD dwSpace, const DWORD dwChannel)
    {
        return b->SetChannel(dwSpace, dwChannel);
    }
}
