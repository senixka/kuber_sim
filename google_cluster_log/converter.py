def dump_yaml_pod_group(fout, arrival_time, pod_count, duration, cpu, memory):
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


def dump_csv_pod_group(fout, arrival_time, pod_count, duration, cpu, memory):
    s = f"{pod_count},{arrival_time},{cpu},{memory},,,,,,constant;{cpu};{memory};{duration}\n"
    fout.write(s)


def dump_state_prolog(fout):
    s = f"""
network_delays:
  api2scheduler: 0
  scheduler2api: 0
  api2kubelet: 1
  kubelet2api: 1
  api2ca: 0
  ca2api: 0

constants:
  kubelet_self_update_period: 100000
  scheduler_self_update_period: 0.5
  monitoring_self_update_period: 10
  scheduler_cycle_max_scheduled: 0
  scheduler_cycle_max_to_try: 0
  unschedulable_queue_period: 30
  ca_self_update_period: 10
  ca_add_node_delay_time: 10
  ca_add_node_min_pending: 0
  ca_remove_node_cpu_percent: 15
  ca_remove_node_memory_percent: 15
  ca_remove_node_delay_cycle: 3
  hpa_self_update_period: 10

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
    s_ext = str(input("Enter format (csv/yaml): ").strip())
    if s_ext not in ["yaml", "csv"]:
        exit(1)

    with open(f"../data/workload/test_g{s_ext}.{s_ext}", 'w') as fout:
        if s_ext == "yaml":
            fout.write("pods:")

        with open("job_and_task.txt", 'r') as fin:
            n_job, n_task = map(int, fin.readline().split())

            for i in range(0, int(n_job)):
                fin.readline()
                arrival_time, group_count = map(int, fin.readline().split())

                for j in range(0, group_count):
                    task_count, duration, cpu, memory = map(int, fin.readline().split())
                    if s_ext == "yaml":
                        dump_yaml_pod_group(fout, arrival_time, task_count, duration, cpu, memory)
                    elif s_ext == "csv":
                        dump_csv_pod_group(fout, arrival_time, task_count, duration, cpu, memory)
                if i % 10000 == 0:
                    print(f"Done job: {i}/{n_job}")

    with open("../data/cluster_state/test_gcsv.yaml", 'w') as fout:
        dump_state_prolog(fout)
        fout.write("nodes:")

        with open("machine_orig.txt", 'r') as fin:
            n_machine, n_group = map(int, fin.readline().split())

            for i in range(0, n_group):
                machine_count, cpu, memory = map(int, fin.readline().split())
                dump_machine_group(fout, machine_count, cpu, memory)


if __name__ == "__main__":
    main()
