import matplotlib.pyplot as plt
import matplotlib
import seaborn as sns

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


def build_plots(metrics: dict):
    mTimes = metrics['time']

    # ////////////////////// Pod counters //////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Pod Counters')
    plt.xlabel('Time (in seconds)')
    plt.ylabel('Pod count')

    plt.plot(mTimes, metrics['pending'], label='Pending pod')
    plt.plot(mTimes, metrics['running'], label='Running pod')
    plt.plot(mTimes, metrics['succeed'], label='Succeed pod')
    plt.plot(mTimes, metrics['failed'], label='Failed pod')
    plt.plot(mTimes, metrics['evicted'], label='Evicted pod')
    plt.plot(mTimes, metrics['removed'], label='Removed pod')
    plt.plot(mTimes, metrics['preempted'], label='Preempted pod')

    plt.legend()
    plt.tight_layout()
    plt.savefig('pod_counters.png')
    plt.close()

    # ////////////////////// Node counter //////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Node Counter')
    plt.xlabel('Time (in seconds)')
    plt.ylabel('Node count')

    plt.plot(mTimes, metrics['nodes'], label='Nodes')

    plt.legend()
    plt.tight_layout()
    plt.savefig('node_counter.png')
    plt.close()

    # ////////////////////// Utilization percent //////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Utilization percent')
    plt.xlabel('Time (in seconds)')
    plt.ylabel('Div')

    # plt.plot(mTimes, metrics['total_cpu'], label='Total cpu')
    # plt.plot(mTimes, metrics['total_memory'], label='Total memory')
    plt.plot(mTimes, metrics['scheduler_used_cpu_percent'], label='Scheduler used cpu')
    plt.plot(mTimes, metrics['scheduler_used_memory_percent'], label='Scheduler used memory')
    plt.plot(mTimes, metrics['kubelets_used_cpu_percent'], label='Kubelets used cpu')
    plt.plot(mTimes, metrics['kubelets_used_memory_percent'], label='Kubelets used memory')
    # plt.plot(mTimes, metrics['nodes'], label='Nodes')

    plt.legend()
    plt.tight_layout()
    plt.savefig('utilization_percent.png')
    plt.close()


    # ////////////////////// Utilization units //////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Utilization')
    plt.xlabel('Time (in seconds)')
    plt.ylabel('Units')

    plt.plot(mTimes, metrics['total_cpu'], label='Total cpu')
    plt.plot(mTimes, metrics['total_memory'], label='Total memory')
    plt.plot(mTimes, metrics['scheduler_used_cpu'], label='Scheduler used cpu')
    plt.plot(mTimes, metrics['scheduler_used_memory'], label='Scheduler used memory')
    plt.plot(mTimes, metrics['kubelets_used_cpu'], label='Kubelets used cpu')
    plt.plot(mTimes, metrics['kubelets_used_memory'], label='Kubelets used memory')
    # plt.plot(mTimes, metrics['nodes'], label='Nodes')

    plt.legend()
    plt.tight_layout()
    plt.savefig('utilization_units.png')
    plt.close()

    # ////////////////////// Utilization Diff //////////////////////

    plt.figure(figsize=(12, 5))
    plt.title('Utilization difference')
    plt.xlabel('Time (in seconds)')
    plt.ylabel('Div')

    plt.plot(mTimes, metrics["cpu_diff_percent"], label='Cpu: Kubelets / Scheduler')
    plt.plot(mTimes, metrics["memory_diff_percent"], label='Memory: Kubelets / Scheduler')

    plt.legend()
    plt.tight_layout()
    plt.savefig('utilization_diff.png')
    plt.close()



def main():
    file_path = input("Enter file path: ").strip()
    metrics = get_metrics_from_file(file_path)
    metrics = process_metrics(metrics)
    build_plots(metrics)


if __name__ == '__main__':
    main()
