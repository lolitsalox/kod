#pragma once

#include <exception>
#include "KodStatus.hpp"

class KodException : public std::exception
{
public:
    KodException(KodStatus status, uint32_t additional_info = 0) :
        m_status(status),
        m_additional_info(additional_info)
    {}
    virtual ~KodException() = default;
    KodException(const KodException&) = default;
    KodException& operator=(const KodException&) = default;
    KodException(KodException&&) = default;
    KodException& operator=(KodException&&) = default;

    KodStatus get_status() const { return m_status; }
    uint32_t get_additional_info() const { return m_additional_info; }

private:
    KodStatus m_status;
    uint32_t m_additional_info;
};
