#include "crc.hpp"

namespace crc
{
    uint8_t calculate(const uint8_t *arr, size_t len)
    {
        uint8_t crc = 0;

        for (size_t i = 0; i < len; i++)
        {
            crc = pgm_read_byte(&table[crc ^ arr[i]]);
        }

        return crc;
    }
}