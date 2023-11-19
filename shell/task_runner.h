#ifndef ISABEL_TASK_RUNNER_H
#define ISABEL_TASK_RUNNER_H

#include <atomic>
#include <chrono>
#include <functional>
#include <mutex>
#include <queue>
#include <thread>

#include "shell/flutter/embedder.h"

namespace shell {

typedef uint64_t (*CurrentTimeFunc)();

class TaskRunner {
public:
  using TaskTimePoint = std::chrono::steady_clock::time_point;
  using TaskFireCallback = std::function<void(const FlutterTask *)>;
  using TaskClosure = std::function<void()>;

  TaskRunner(CurrentTimeFunc, TaskFireCallback);
  ~TaskRunner();

  bool runs_tasks_on_current_thread();

  void post_delayed_task(FlutterTask, uint64_t);
  void post_closure(TaskClosure);

  void process();

private:
  typedef std::variant<FlutterTask, TaskClosure> TaskVariant;

  struct Task {
    uint64_t order;
    TaskTimePoint fire_time;
    TaskVariant variant;

    struct Comparer {
      bool operator()(const Task &a, const Task &b) {
        if (a.fire_time == b.fire_time) {
          return a.order > b.order;
        }
        return a.fire_time > b.fire_time;
      }
    };
  };

  std::atomic_uint64_t m_global_task_order{0};

  void enqueue_task(Task);

  TaskRunner::TaskTimePoint time_point_from_flutter(uint64_t delay_time);

  TaskFireCallback m_fire_callback;
  CurrentTimeFunc m_get_current_time;

  std::thread::id m_main_thread_id;

  std::mutex m_task_queue_mutex;
  std::priority_queue<Task, std::deque<Task>, Task::Comparer> m_task_queue;
};

} // namespace shell

#endif // ISABEL_TASK_RUNNER_H
