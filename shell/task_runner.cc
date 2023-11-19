#include "shell/task_runner.h"
#include "shell/log.h"

namespace shell {

TaskRunner::TaskRunner(CurrentTimeFunc current_time,
                       TaskRunner::TaskFireCallback callback)
    : m_fire_callback(callback), m_get_current_time(current_time),
      m_main_thread_id(std::this_thread::get_id()) {}

bool TaskRunner::runs_tasks_on_current_thread() {
  return std::this_thread::get_id() == m_main_thread_id;
}

void TaskRunner::post_delayed_task(FlutterTask flutter_task,
                                   uint64_t delay_time) {
  Task task;
  task.fire_time = time_point_from_flutter(delay_time);
  task.variant = flutter_task;

  enqueue_task(task);
}

void TaskRunner::process() {
  const TaskTimePoint now = TaskTimePoint::clock::now();
  std::vector<Task> expired_tasks;

  // Process expired tasks.
  {
    std::lock_guard<std::mutex> lock(m_task_queue_mutex);
    while (!m_task_queue.empty()) {
      const auto &top = m_task_queue.top();
      // If this task (and all tasks after this) has not yet expired, there is
      // nothing more to do. Quit iterating.
      if (top.fire_time > now) {
        break;
      }

      // Make a record of the expired task. Do NOT service the task here
      // because we are still holding onto the task queue mutex. We don't want
      // other threads to block on posting tasks onto this thread till we are
      // done processing expired tasks.
      expired_tasks.push_back(m_task_queue.top());

      // Remove the tasks from the delayed tasks queue.
      m_task_queue.pop();
    }
  }

  // Fire expired tasks.
  {
    // Flushing tasks here without holing onto the task queue mutex.
    for (const auto &task : expired_tasks) {
      if (auto flutter_task = std::get_if<FlutterTask>(&task.variant)) {
        m_fire_callback(flutter_task);
      } else if (auto closure = std::get_if<TaskClosure>(&task.variant))
        (*closure)();
    }
  }
}

void TaskRunner::enqueue_task(shell::TaskRunner::Task task) {
  task.order = m_global_task_order++;
  std::lock_guard<std::mutex> lock(m_task_queue_mutex);
  m_task_queue.push(task);
}

TaskRunner::TaskTimePoint
TaskRunner::time_point_from_flutter(uint64_t delay_time) {
  const auto now = TaskRunner::TaskTimePoint::clock::now();
  const auto flutter_duration = delay_time - m_get_current_time();
  return now + std::chrono::nanoseconds(flutter_duration);
}

TaskRunner::~TaskRunner() {}
} // namespace shell