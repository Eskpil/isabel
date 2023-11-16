#ifndef ISABEL_SHELL_LOG_H_
#define ISABEL_SHELL_LOG_H_

#include <iostream>
#include <format>
#include <format>
#include <fstream>
#include <string_view>
#include <source_location>

namespace shell::log {

enum LogLevel { Error, Info, Warn, Debug, Fatal };

template <typename... Args>
void log_internal(LogLevel level, const std::format_string<Args...> fmt, Args &&... args) {
  switch (level) {
     case Fatal:
       std::cout << "\x1B[31m"
                 << "[Fatal] " << "\x1B[0m" << std::vformat(fmt.get(), std::make_format_args(args...))
                 << std::endl;
       break;
     case Error:
       std::cout << "\x1B[31m"
                 << "[Error] " << "\x1B[0m" << std::vformat(fmt.get(), std::make_format_args(args...))
                 << std::endl;
       break;
     case Info:
       std::cout << "\x1B[34m"
                 << "[Info] " << "\x1B[0m" << std::vformat(fmt.get(), std::make_format_args(args...))
                 << std::endl;
       break;
     case Warn:
       std::cout << "\x1B[32m"
                 << "[Warn] " << "\x1B[0m" << std::vformat(fmt.get(), std::make_format_args(args...))
                 << std::endl;
       break;
     case Debug:
      std::cout << "\x1B[35m"
                << "[Debug] " << "\x1B[0m" << std::vformat(fmt.get(), std::make_format_args(args...))
                << std::endl;
      break;
    }
  }

template <typename... Args>
inline void error(std::format_string<Args...> fmt, Args &&...args) {
  log_internal(LogLevel::Error, fmt, std::forward<Args>(args)...);
}

template <typename... Args> inline void dbg(std::format_string<Args...> fmt, Args &&...args) {
  log_internal(LogLevel::Debug, fmt, std::forward<Args>(args)...);
};

template <typename... Args> inline void info(std::format_string<Args...> fmt, Args &&...args) {
  log_internal(LogLevel::Info, fmt, std::forward<Args>(args)...);
};

template <typename... Args> inline void warn(std::format_string<Args...> fmt, Args &&...args) {
  log_internal(LogLevel::Warn, fmt, std::forward<Args>(args)...);
};

template <typename... Args> inline void fatal(std::format_string<Args...> fmt, Args &&...args) {
  log_internal(LogLevel::Fatal, fmt, std::forward<Args>(args)...);
  exit(EXIT_FAILURE);
};
}

#endif // ISABEL_SHELL_LOG_H_