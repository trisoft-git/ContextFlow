# 📋 ContextFlow Task 및 Phase 생성 가이드

이 문서는 ContextFlow 프로젝트에서 새로운 기능 구현이나 대규모 변경 작업을 위한 `task.md` 및 `phase-X.md` 파일 생성 규약을 정의합니다.

## 1. 디렉토리 구조
모든 태스크 관련 파일은 프로젝트 루트의 `.tasks/{task_name}/` 디렉토리에 위치합니다.

```
.tasks/{task_name}/
├── index.json          # 전체 태스크 상태 및 페이즈 관리
├── task.md             # 전체 태스크 개요 및 최종 목표
└── phases/
    ├── phase-1.md      # 페이즈 1 상세 설계 및 작업 목록
    ├── phase-2.md      # 페이즈 2 상세 설계 및 작업 목록
    └── ...
```

## 2. 파일 형식 규정

### 2.1 index.json
태스크의 현재 진행 상황을 기계가 읽을 수 있는 형식으로 기록합니다.
```json
{
  "task_name": "Task Name",
  "current_phase": 1,
  "total_phases": 3,
  "status": "in_progress",
  "error_message": null,
  "phases": [
    {"id": 1, "name": "Phase Name", "status": "pending"},
    ...
  ]
}
```

### 2.2 task.md
태스크의 목적, 배경, 그리고 각 페이즈별 핵심 목표를 기술합니다.
- `[ ]` 형식을 사용하여 전체 진행도를 시각화합니다.

### 2.3 phase-X.md
해당 페이즈에서 수행할 구체적인 작업 목록(TODO)을 정의합니다.
- 각 작업은 안티그래비티 에이전트가 추적할 수 있도록 명확한 체크리스트 형태로 작성합니다.

## 3. 생성 절차
1. 사용자로부터 구현 계획(Draft) 승인을 받는다.
2. 위 구조에 맞춰 디렉토리와 파일들을 생성한다.
3. 생성 완료 후 사용자에게 `task.md` 위치를 보고하고 실행 준비가 되었음을 알린다.
