import streamlit as st
import tools
import random
import matplotlib.pyplot as plt
from matplotlib.patches import Patch
import numpy as np
#import scenarios as sc # in dev 

import plotly.graph_objects as go
import plotly.express as px



def plot_history_matrix(history, scheduler):
    STATE_COLORS = {
        0: "lightgray",
        1: "#e67e22",
        2: "#3498db",
        3: "#2ecc71",
        -1: "white"
    }

    CORE_COLORS = [
        "rgba(255, 200, 200, 0.2)",  # red 
        "rgba(200, 255, 200, 0.2)",  # green light
        "rgba(200, 200, 255, 0.2)",  # Blue light
        "rgba(255, 255, 200, 0.2)",  # yellow 
        "rgba(255, 200, 255, 0.2)",  # purple light
    ]

    num_cores = len(scheduler.cores)
    num_ticks = min(scheduler.time, len(history))
    
    fig = go.Figure()
    
    y_start = 0
    
    for core_id in range(num_cores):
        # take task of each cores 
        tasks_in_core = set()
        for tick in range(num_ticks):
            tasks_in_core.update(history[tick][core_id]["task_states"].keys())
        
        sorted_tasks = sorted(tasks_in_core)
        y_end = y_start + len(sorted_tasks) - 1
        
        # Add colored rectangle to separate cores 
        if sorted_tasks:
            fig.add_hrect(
                y0=y_start - 0.5,
                y1=y_end + 0.5,
                fillcolor=CORE_COLORS[core_id % len(CORE_COLORS)],
                line_width=1,
                line_color="rgba(0,0,0,0.3)",
                layer="below"
            )
        
        # Afficher les tâches
        for task_id in sorted_tasks:
            states = []
            for tick in range(num_ticks):
                task_states = history[tick][core_id]["task_states"]
                states.append(task_states.get(task_id, -1))
            
            fig.add_trace(go.Scatter(
                x=list(range(num_ticks)),
                y=[f"Core {core_id} - T{task_id}"] * num_ticks,
                mode="markers",
                marker=dict(
                    color=[STATE_COLORS[s] for s in states],
                    size=18,
                    symbol="square",
                    line=dict(width=1, color="gray")
                ),
                showlegend=False
            ))
            
            y_start += 1

    # Légende
    for label, color in [("READY", "lightgray"), ("BLOCKED", "#e67e22"),
                          ("RUNNING", "#3498db")]:
        fig.add_trace(go.Scatter(
            x=[None], y=[None], mode="markers",
            marker=dict(color=color, size=12, symbol="square"),
            name=label
        ))

    fig.update_layout(
        title="State of tasks per core over time",
        xaxis_title="Tick",
        yaxis_title="Core / Task",
        height=100 + 40 * scheduler.total_tasks,
        xaxis=dict(tickmode='linear', dtick=1)
    )
    return fig

def plot_tasks(tasks):
    fig, ax = plt.subplots()
    
    # Palette  colors per core
    core_color_map = {}
    colors = ['blue', 'green', 'red', 'orange', 'purple', 'brown', 'pink', 'cyan']
    
    for task in tasks:
        if task.core_assigned:
            core_id = task.core_assigned.id
            if core_id not in core_color_map:
                core_color_map[core_id] = colors[len(core_color_map) % len(colors)]
            color = core_color_map[core_id]
        else:
            color = 'gray'
        
        ax.barh(f"T{task.id}", task.time_remaining, color=color)
    

    
    legend_elements = [Patch(facecolor=color, label=f'Core {core_id}') 
                       for core_id, color in core_color_map.items()]
    if legend_elements:
        ax.legend(handles=legend_elements)
    
    ax.set_xlabel("Time Remaining")
    ax.set_title("Task Time Remaining")
    st.pyplot(fig)

def load_update():
        for core in st.session_state.scheduler.cores:
            core.load = (len(core.queue) + (1 if core.current_task else 0)) / st.session_state.scheduler.total_tasks if st.session_state.scheduler.total_tasks > 0 else 0

def plot_load():
    fig, ax = plt.subplots()
    for core in st.session_state.scheduler.cores:
        ax.bar(f"Core {core.id}", core.load, color="orange")
    ax.set_ylim(0, 1)
    ax.set_ylabel("Load")
    ax.set_title("Core Load")
    st.pyplot(fig)

def plot_load_history(history, scheduler):

    fig = go.Figure()
    colors = px.colors.qualitative.Set2

    for core_id in range(len(scheduler.cores)):

        loads = []

        for tick in range(len(history)):
            core_state = history[tick][core_id]

            load = (
                len(core_state["queue"])
                + (1 if core_state["current"] is not None else 0)
            ) / max(scheduler.total_tasks, 1)

            loads.append(load)

        fig.add_trace(
            go.Scatter(
                x=list(range(len(history))),
                y=loads,
                mode="lines",
                name=f"Core {core_id}",
                line=dict(
                    width=3,
                    color=colors[core_id % len(colors)]
                ),
                hovertemplate=
                    "Tick %{x}<br>"
                    "Load %{y:.2f}<extra></extra>"
            )
        )

    fig.update_layout(
        title="CPU Load Evolution",
        xaxis_title="Tick",
        yaxis_title="Normalized Load",
        template="plotly_white",
        hovermode="x unified",
        height=500,
        legend=dict(
            orientation="h",
            y=1.1
        )
    )

    st.plotly_chart(fig, use_container_width=True)

def plot_load_heatmap(history, scheduler):

    num_cores = len(scheduler.cores)
    num_ticks = len(history)

    heatmap_data = []

    for core_id in range(num_cores):
        core_loads = []

        for tick in range(num_ticks):
            core_state = history[tick][core_id]

            load = (
                len(core_state["queue"])
                + (1 if core_state["current"] is not None else 0)
            ) / max(scheduler.total_tasks, 1)

            core_loads.append(load)

        heatmap_data.append(core_loads)

    fig = go.Figure(
        data=go.Heatmap(
            z=heatmap_data,
            x=list(range(num_ticks)),
            y=[f"Core {i}" for i in range(num_cores)],
            colorscale="RdYlGn_r",
            colorbar=dict(title="Load"),
            hovertemplate=(
                "Core: %{y}<br>"
                "Tick: %{x}<br>"
                "Load: %{z:.2f}<extra></extra>"
            ),
        )
    )

    fig.update_layout(
        title="Core Load Heatmap",
        xaxis_title="Tick",
        yaxis_title="Core",
        height=max(400, 80 * num_cores),
        template="plotly_white",
    )

    st.plotly_chart(fig, use_container_width=True)

def add_task():
    # we choose the core randomly for now
    core_assigned = random.choice(st.session_state.scheduler.cores)

    new_task = tools.Task(id = st.session_state.scheduler.total_tasks+1, burst_time=random.randint(1, 10), core_assigned=core_assigned)

    st.session_state.scheduler.total_tasks += 1
    st.session_state.scheduler.tasks.append(new_task)


    #last we tell the core it has been assigned th task
    st.session_state.scheduler.cores[core_assigned.id].assign_task(new_task)
    load_update()
    

    st.session_state.updated = True # force to load the new state

def do_tick():
    st.session_state.scheduler.tick()
    load_update()
    st.session_state.history.append(tools.snapshot(st.session_state.scheduler.cores))
    st.session_state.updated = True # force load the new state

def reset():
    
    st.session_state.scheduler = tools.initialize_scheduler(st.session_state.num_cores)
    st.session_state.history = []
    st.session_state.updated = True

def main():
    st.title("Task testing for load function")
    st.write("This is a test page to verify that the load function works correctly.")

    # session state init
    if 'scheduler' not in st.session_state:
        st.session_state.num_cores = 3
        st.session_state.scheduler = tools.initialize_scheduler(st.session_state.num_cores)
        st.session_state.history = []
        st.session_state.updated = True
    if 'history' not in st.session_state:
        st.session_state.history = []
    if 'updated' not in st.session_state:
        st.session_state.updated = True
    
    
    scheduler = st.session_state.scheduler
    NUM_TASKS = scheduler.total_tasks
    NUM_CORES = len(scheduler.cores)
    NUM_TICKS = len(st.session_state.history)
    
    tab1, tab2 = st.tabs(["manual test", " scenarios tests?"])
    with tab1 : 
        col1, col2,col3 = st.columns(3)
        with col1:
            st.button("add task",on_click = add_task)
            st.metric(label="tasks", value=NUM_TASKS)

        with col2:
            st.button("tick", on_click=do_tick)
            st.metric(label="ticks", value=NUM_TICKS)

        with col3:
            st.button("reset", on_click= reset)
            st.metric(label="cores", value=NUM_CORES)
        

        col1, col2 = st.columns(2)
        with col1:
            st.header("Task Visualization")
            plot_tasks(scheduler.tasks)
        with col2:
            st.header("Load Visualization")
            plot_load()

        st.plotly_chart(plot_history_matrix(st.session_state.history, scheduler))
        col1, col2 = st.columns(2)
        with col1:
            plot_load_history(st.session_state.history, scheduler)
        with col2:
            plot_load_heatmap(st.session_state.history, scheduler)
        with st.expander("Debug - JSON"):
            debug_data = {
                "scheduler": {
                    "total_tasks": st.session_state.scheduler.total_tasks,
                    "time": st.session_state.scheduler.time,
                    "num_cores": len(st.session_state.scheduler.cores),
                    "tasks": [
                        {
                            "id": t.id, 
                            "burst_time": t.time_remaining,
                            "state": str(t.state),
                            "core_assigned": t.core_assigned.id if t.core_assigned else None
                        } 
                        for t in st.session_state.scheduler.tasks
                    ]
                },
                "history_length": len(st.session_state.history),
                "history_preview": st.session_state.history[-3:] if st.session_state.history else []
            }
            
            st.json(debug_data)

  
   


if __name__ == "__main__":    main()

