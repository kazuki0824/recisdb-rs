//
// Created by maleicacid on 2023/07/14.
//

#include "../IBonDriver.hpp"

extern "C" {
    const BOOL C_SetLnbPower(IBonDriver3 * b, const BOOL bEnable) {
        return b->SetLnbPower(bEnable);
    }
}