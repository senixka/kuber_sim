def dump_yaml_pod_group(fout, arrival_time, pod_count, duration, cpu, memory):
    s = f"""
  - submit_time: {arrival_time}
    event:
      !AddPodGroup
      pod_count: {pod_count}
      pod:
        spec:
          request_cpu: {cpu}
          request_memory: {memory}
          limit_cpu: {memory}
          limit_memory: {memory}
          load:
            !Constant
            cpu: {cpu}
            memory: {memory}
            duration: {duration}
"""
    fout.write(s)


def dump_csv_pod_group(fout, arrival_time, pod_count, duration, cpu, memory):
    s = f"{arrival_time};0;{pod_count};;{{{{}};{{{cpu};{memory};{cpu};{memory};;{{0;{cpu};{memory};{duration}}};{{}};{{}};{{}}}}}};{{}};{{}}\n"
    fout.write(s)


def dump_state_prolog(fout):
    s = f"""
monitoring:
  self_update_period: 1

scheduler:
  self_update_period: 1
  unschedulable_queue_backoff_delay: 5

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

    with open("in_config.yaml", 'w') as fout:
        dump_state_prolog(fout)
        fout.write("nodes:")

        with open("machine_orig.txt", 'r') as fin:
            n_machine, n_group = map(int, fin.readline().split())

            for i in range(0, n_group):
                machine_count, cpu, memory = map(int, fin.readline().split())
                dump_machine_group(fout, machine_count, cpu, memory)

    with open(f"in_trace.{s_ext}", 'w') as fout:
        if s_ext == "yaml":
            fout.write("trace:")

        with open("job_and_task.txt", 'r') as fin:
            n_job, n_task = map(int, fin.readline().split())

            for i in range(0, int(n_job)):
                fin.readline()
                arrival_time, group_count = map(int, fin.readline().split())

                for _ in range(0, group_count):
                    task_count, duration, cpu, memory = map(int, fin.readline().split())
                    if duration == 4294967295:
                        duration = 60 * 60 * 24 * 29
                    if s_ext == "yaml":
                        dump_yaml_pod_group(fout, arrival_time, task_count, duration, cpu, memory)
                    elif s_ext == "csv":
                        dump_csv_pod_group(fout, arrival_time, task_count, duration, cpu, memory)
                if i % 50000 == 0:
                    print(f"Done job: {i}/{n_job}")


if __name__ == "__main__":
    main()
