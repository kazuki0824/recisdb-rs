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
}