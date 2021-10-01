/*
  LibISDB
  Copyright(c) 2017-2020 DBCTRADO

  This program is free software; you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation; either version 2 of the License, or
  (at your option) any later version.

  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program; if not, write to the Free Software
  Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
*/

/**
 @file   BonDriver.hpp
 @brief  BonDriver
 @author DBCTRADO
*/


#ifndef LIBISDB_BON_DRIVER_H
#define LIBISDB_BON_DRIVER_H

#include <stdint.h>

/** IBonDriver インターフェース */
class IBonDriver
{
public:
#ifndef LIBISDB_WINDOWS
	typedef int BOOL;
	typedef uint8_t BYTE;
	typedef uint32_t DWORD;
#endif

	virtual const BOOL OpenTuner();
	virtual void CloseTuner();

	virtual const BOOL SetChannel(const BYTE bCh);
	virtual const float GetSignalLevel();

	virtual const DWORD WaitTsStream(const DWORD dwTimeOut = 0);
	virtual const DWORD GetReadyCount();

	virtual const BOOL GetTsStream(BYTE *pDst, DWORD *pdwSize, DWORD *pdwRemain);
	virtual const BOOL GetTsStream(BYTE **ppDst, DWORD *pdwSize, DWORD *pdwRemain);

	virtual void PurgeTsStream();

	virtual void Release();
};

/** IBonDriver2 インターフェース */
class IBonDriver2 : public IBonDriver
{
public:
#ifdef LIBISDB_WINDOWS
	typedef WCHAR CharType;
#else
	typedef char16_t CharType;
	typedef const CharType * LPCTSTR;
#endif

	virtual LPCTSTR GetTunerName();

	virtual const BOOL IsTunerOpening();

	virtual const LPCTSTR EnumTuningSpace(const DWORD dwSpace);
	virtual const LPCTSTR EnumChannelName(const DWORD dwSpace, const DWORD dwChannel);

	virtual const BOOL SetChannel(const DWORD dwSpace, const DWORD dwChannel);

	virtual const DWORD GetCurSpace();
	virtual const DWORD GetCurChannel();

// IBonDriver
	virtual void Release() override;
};

/** IBonDriver3 インターフェース */
class IBonDriver3 : public IBonDriver2
{
public:
	virtual const DWORD GetTotalDeviceNum();
	virtual const DWORD GetActiveDeviceNum();
	virtual const BOOL SetLnbPower(const BOOL bEnable);

// IBonDriver
	virtual void Release() override;
};

extern "C" IBonDriver * CreateBonDriver();

#endif	// ifndef LIBISDB_BON_DRIVER_H