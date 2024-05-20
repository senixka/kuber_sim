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
            duration: {duration}"""
    fout.write(s)


def dump_csv_pod_group(fout, arrival_time, pod_count, duration, cpu, memory):
    s = f"{arrival_time};0;{pod_count};;{{{{}};{{{cpu};{memory};{cpu};{memory};;{{0;{cpu};{memory};{duration}}};{{}};{{}};{{}}}}}};{{}};{{}}\n"
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


def dump_state_prolog(fout):
    s = f"""
monitoring:
  self_update_period: 1

scheduler:
  self_update_period: 0.5
  unschedulable_queue_backoff_delay: 30

"""
    fout.write(s)


def main():
    # Get output format
    out_extension = str(input('Enter output trace format (csv/yaml): ').strip())
    if out_extension not in ['yaml', 'csv']:
        print(f'Bad format "{out_extension}". Format should be "yaml" or "csv"')
        exit(1)

    # Output cluster config.yaml
    with open('./data_out/config.yaml', 'w') as fout:
        # Write global constants
        dump_state_prolog(fout)

        # Write nodes
        fout.write('nodes:')
        with open("./data_in/machine_orig.txt", 'r') as fin:
            # Get node group count
            _, n_group = map(int, fin.readline().split())
            for _ in range(0, n_group):
                # Get node group node template
                machine_count, cpu, memory = map(int, fin.readline().split())
                # Dump node
                dump_machine_group(fout, machine_count, cpu, memory)

    # Output cluster trace.yaml or trace.csv
    with open(f"./data_out/in_trace.{out_extension}", 'w') as fout:
        if out_extension == "yaml":
            fout.write("trace:")

        # Read input trace data
        with open("./data_in/job_and_task.txt", 'r') as fin:
            pod_group_count, _ = map(int, fin.readline().split())

            # Process all pod groups in job
            for i in range(int(pod_group_count)):
                fin.readline() # Skip empty line
                arrival_time, group_count = map(int, fin.readline().split())

                # Generate all pods from this group
                for _ in range(group_count):
                    pod_count, duration, cpu, memory = map(int, fin.readline().split())
                    # Shift max time to trace len (29 days)
                    if duration == 4294967295:
                        duration = 60 * 60 * 24 * 29
                    # Dump pod
                    if out_extension == "yaml":
                        dump_yaml_pod_group(fout, arrival_time, pod_count, duration, cpu, memory)
                    elif out_extension == "csv":
                        dump_csv_pod_group(fout, arrival_time, pod_count, duration, cpu, memory)
                # Prints current progress
                if i % 100000 == 0:
                    print(f"Done pod_group_count: {i}/{pod_group_count}")


if __name__ == "__main__":
    main()
