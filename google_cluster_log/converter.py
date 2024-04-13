def dump_pod_group(fout, arrival_time, pod_count, duration, cpu, memory):
    s = f"""
  - pod_group:
    amount: {pod_count}
    pod:
      spec:
        arrival_time: {arrival_time}
        request_cpu: {cpu}
        request_memory: {memory}
        load:
          !Constant
          cpu: {cpu}
          memory: {memory}
          duration: {duration}"""
    fout.write(s)


def dump_state_prolog(fout):
    s = f"""
network_delays:
  api2scheduler: 0
  scheduler2api: 0
  api2kubelet: 0
  kubelet2api: 0

constants:
  kubelet_self_update_period: 2
  scheduler_self_update_period: 10
  monitoring_self_update_period: 2

"""
    fout.write(s)


def dump_machine_group(fout, machine_count, cpu, memory):
    s = f"""
  - node_group:
    amount: {machine_count}
    node:
      spec:
        installed_cpu: {cpu}
        installed_memory: {memory}"""
    fout.write(s)


def main():
    with open("../data/workload/pods.yaml", 'w') as fout:
        fout.write("pods:")

        with open("job_and_task.txt", 'r') as fin:
            n_job, n_task = map(int, fin.readline().split())

            for i in range(0, int(n_job)):
                fin.readline()
                arrival_time, group_count = map(int, fin.readline().split())

                for j in range(0, group_count):
                    task_count, duration, cpu, memory = map(int, fin.readline().split())
                    if True or 50000 < i < 200000:
                        dump_pod_group(fout, arrival_time, task_count, duration, cpu, memory)
                if i % 10000 == 0:
                    print(f"Done job: {i}/{n_job}")

    with open("../data/cluster_state/state.yaml", 'w') as fout:
        dump_state_prolog(fout)
        fout.write("nodes:")

        with open("machine_orig.txt", 'r') as fin:
            n_machine, n_group = map(int, fin.readline().split())

            for i in range(0, n_group):
                machine_count, cpu, memory = map(int, fin.readline().split())
                dump_machine_group(fout, machine_count, cpu, memory)


if __name__ == "__main__":
    main()
