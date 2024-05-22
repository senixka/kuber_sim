# Примеры использования KuberSim

Для запуска, перейдите в нужную директорию с примером и выполните:
```sh
cargo run
```

#### Примеры базовой функциональности:
- `basic` - базовый пример запуска простой симуляции.
- `eviction` - демонстрирует работу механизма выселения подов.
- `failed` - демонстрирует работу лимитов к ресурсам у подов.
- `pod_group_removal` - демонстрирует работу механизма удаления группы подов из симуляции.
- `monitoring` - демонстрирует работу и возможности сбора метрик в симуляции.
- `csv_trace` - демонстрирует поддержку чтения трейсов в CSV формате.
- `multithread` - демонстрирует работу многопоточных симуляций.

#### Примеры моделей нагрузки пода:
- `load_type_constant` - пример работы модели нагрузки Constant.
- `load_type_constant_infinite` - пример работы модели нагрузки ConstantInfinite.
- `load_type_busybox` - пример работы модели нагрузки BusyBox.
- `load_type_busybox_infinite` - пример работы модели нагрузки BusyBoxInfinite.

#### Примеры конвейера планировщика:
- `scheduler_filter_node_selector` - поддержка селекторов узлов.
- `scheduler_filter_node_affinity` - поддержка Affinity/Anti-affinity правил с эффектов NoSchedule.
- `scheduler_score_node_affinity` - поддержка оценки узлов на основе Affinity/Anti-affinity правил с эффектом PreferNoSchedule.

#### Примеры c Cluster Autoscaler:
- `ca_basic` - базовый пример работы CA.
- `ca_basic_with_group_remove` - показывает реакцию CA на удаление группы, за которой он следил.

#### Примеры c Horizontal Pod Autoscaler:
- `hpa_basic` - базовый пример работы HPA.
- `hpa_basic_with_group_remove` - показывает реакцию HPA на удаление группы, за которой он следил.

#### Примеры c Vertical Pod Autoscaler:
- `vpa_basic` - базовый пример работы VPA.
- `vpa_basic_with_group_remove` - показывает реакцию VPA на удаление группы, за которой он следил.
- `vpa_failed` - пример показывает, что VPA умеет перезапускать поды, которые перешли в состояние failed.
