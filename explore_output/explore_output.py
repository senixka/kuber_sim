import matplotlib.pyplot as plt
import matplotlib
import seaborn as sns
from os.path import isfile, join
import os

sns.set_theme()
matplotlib.use('Agg')
plt.ioff()


def number_type(value: str):
    if value.count('.') > 0:
        return float(value)
    return int(value)


def smart_percent(a, b):
    a, b = float(a), float(b)
    if b != 0:
        return a / b
    return 0


def get_metrics_from_file(input_file_path: str):
    metrics = {}

    with open(input_file_path, 'r') as fin:
        names = list(fin.readline().strip().split(','))
        for name in names:
            metrics[name] = list()

        for line in fin:
            line = line.strip()
            if line == "":
                break
            values = list(map(number_type, line.split(',')))
            for name, value in zip(names, values):
                metrics[name].append(value)

    return metrics


def process_metrics(metrics: dict):
    metrics["scheduler_used_cpu_percent"] = [smart_percent(a, b) for a, b in zip(metrics['scheduler_used_cpu'], metrics['total_cpu'])]
    metrics["scheduler_used_memory_percent"] = [smart_percent(a, b) for a, b in zip(metrics['scheduler_used_memory'], metrics['total_memory'])]
    metrics["kubelets_used_cpu_percent"] = [smart_percent(a, b) for a, b in zip(metrics['kubelets_used_cpu'], metrics['total_cpu'])]
    metrics["kubelets_used_memory_percent"] = [smart_percent(a, b) for a, b in zip(metrics['kubelets_used_memory'], metrics['total_memory'])]
    metrics["cpu_diff"] = [a - b for a, b in zip(metrics['kubelets_used_cpu'], metrics['scheduler_used_cpu'])]
    metrics["memory_diff"] = [a - b for a, b in zip(metrics['kubelets_used_memory'], metrics['scheduler_used_memory'])]
    metrics["cpu_diff_percent"] = [smart_percent(a, b) for a, b in zip(metrics['kubelets_used_cpu'], metrics['scheduler_used_cpu'])]
    metrics["memory_diff_percent"] = [smart_percent(a, b) for a, b in zip(metrics['kubelets_used_memory'], metrics['scheduler_used_memory'])]
    return metrics


def build_plots(metrics: dict, prefix: str):
    mTimes = metrics['time']

    # ////////////////////// Pod counters //////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Number of pods by phase')
    plt.xlabel('Time in simulation (s)')
    plt.ylabel('Pod count')

    plt.plot(mTimes, metrics['pending'], label='Pending')
    plt.plot(mTimes, metrics['running'], label='Running')
    plt.plot(mTimes, metrics['succeed'], label='Succeed pods')
    plt.plot(mTimes, metrics['failed'], label='Failed')
    plt.plot(mTimes, metrics['evicted'], label='Evicted')
    plt.plot(mTimes, metrics['removed'], label='Removed')
    plt.plot(mTimes, metrics['preempted'], label='Preempted')

    plt.legend()
    plt.tight_layout()
    plt.savefig(prefix + '_pod_counters.png')
    plt.close()

    # ////////////////////// Node counter //////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Number of running nodes on cluster')
    plt.xlabel('Time in simulation (s)')
    plt.ylabel('Node count')

    plt.plot(mTimes, metrics['nodes'], label='Nodes')

    plt.legend()
    plt.tight_layout()
    plt.savefig(prefix + '_node_counter.png')
    plt.close()

    # ////////////////////// Node utilization percent //////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Overall utilization of nodes')
    plt.xlabel('Time in simulation (s)')
    plt.ylabel('Actual consumption / Total available')

    plt.plot(mTimes, metrics['kubelets_used_cpu_percent'], label='Cpu')
    plt.plot(mTimes, metrics['kubelets_used_memory_percent'], label='Memory')

    plt.legend()
    plt.tight_layout()
    plt.savefig(prefix + '_utilization_percent.png')
    plt.close()


    # ///////////////////////////////// Node utilization units //////////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Overall utilization of nodes')
    plt.xlabel('Time in simulation (s)')
    plt.ylabel('Resource units')

    plt.plot(mTimes, metrics['total_cpu'], label='Total cpu')
    plt.plot(mTimes, metrics['total_memory'], label='Total memory')
    plt.plot(mTimes, metrics['kubelets_used_cpu'], label='Used cpu')
    plt.plot(mTimes, metrics['kubelets_used_memory'], label='Used memory')

    plt.legend()
    plt.tight_layout()
    plt.savefig(prefix + '_utilization_units.png')
    plt.close()

    # ////////////////////// Utilization scheduler vs kubelets diff /////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('The ratio of actual pods consumption to their requests')
    plt.xlabel('Time in simulation (s)')
    plt.ylabel('Actual consumption / Request')

    plt.plot(mTimes, metrics["cpu_diff_percent"], label='Cpu')
    plt.plot(mTimes, metrics["memory_diff_percent"], label='Memory')

    plt.legend()
    plt.tight_layout()
    plt.savefig(prefix + '_utilization_diff.png')
    plt.close()


def main():
    # Get all files in input directory
    template_files = [f for f in os.listdir("./data_in/") if isfile(join("./data_in/", f))]
    # For each template file generate trace
    for file_name in template_files:
        # Read file with metrics
        metrics = get_metrics_from_file('./data_in/' + file_name)
        # Count extra metrics
        metrics = process_metrics(metrics)
        # Build plots
        build_plots(metrics, './data_out/' + os.path.splitext(os.path.basename(file_name))[0])


if __name__ == '__main__':
    main()
