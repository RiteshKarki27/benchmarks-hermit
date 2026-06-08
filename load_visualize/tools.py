import streamlit
from enum import Enum
import random 

class State(Enum):
    READY = 0
    BLOCKED = 1
    RUNNING = 2
    FINISHED = 3

class Scheduler:
    def __init__(self, cores):
        self.cores = cores
        self.time = 0
        self.total_tasks = 0
        self.tasks = []

    def tick(self):
        for core in self.cores:
            core.tick()
        self.time += 1


class Task:
    def __init__(self,id,burst_time,core_assigned=None,state=State.READY):
        self.id = id
        self.state = state
        self.time_remaining = burst_time
        self.core_assigned = core_assigned
    
    def run(self):
        if self.state == State.READY:
            self.state = State.RUNNING
        if self.state == State.RUNNING:
            self.time_remaining -= 1
            if self.time_remaining <= 0:
                self.state = State.FINISHED
    
class core:
    def __init__(self, id):
        self.id = id
        self.current_task = None
        self.queue = [] 
        self.load = 0

    def assign_task(self, task):
        if self.current_task is None:
            self.current_task = task
            task.state = State.RUNNING
        else:
            self.queue.append(task)  # queue if occupied
            task.state = State.READY
            

    def tick(self):
        if self.current_task:
            self.current_task.run()
            if self.current_task.state == State.FINISHED:
                self.current_task = self.queue.pop(0) if self.queue else None
                if self.current_task:
                    self.current_task.state = State.RUNNING
                    


def initialize_tasks(num_tasks):
    tasks = []
    for i in range(num_tasks):
        burst_time = random.randint(1, 10)
        tasks.append(Task(i, burst_time))
    return tasks

def initialize_cores(num_cores):
    cores = []
    for i in range(num_cores):
        cores.append(core(i))
    return cores

def initialize_scheduler(num_cores):
    cores = [core(i) for i in range(num_cores)]
    scheduler = Scheduler(cores)
    scheduler.total_tasks = 0
    scheduler.tasks = []
    return scheduler

    
def snapshot(cores): # snapshot of the current state of all cores and their tasks
    snapshot = []
    for core in cores:
        snapshot.append({
            "current": core.current_task.id if core.current_task else None,
            "queue": [t.id for t in core.queue],
            "task_states": {
                t.id: t.state.value
                for t in ([core.current_task] if core.current_task else []) + core.queue
            }
        })
    return snapshot



