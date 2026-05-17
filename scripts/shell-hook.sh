# ContextFlow Bash/Zsh Hook
_contextflow_hook() {
    local EXIT_CODE=$?
    # 마지막 명령어 추출 (번호 제외)
    local LAST_CMD=$(history 1 | sed 's/^[ ]*[0-9]*[ ]*//')
    
    # 무한 루프 방지 및 특정 명령어 제외
    if [[ "$LAST_CMD" == *"contextflow"* ]] || [[ -z "$LAST_CMD" ]]; then
        return
    fi

    # Python을 활용하여 JSON 인코딩 안전화 (특수문자 및 따옴표 오염 방지)
    local PAYLOAD=""
    if command -v python3 >/dev/null 2>&1; then
        PAYLOAD=$(python3 -c '
import sys, json
print(json.dumps({
    "type": "terminal_command",
    "content": sys.argv[1],
    "metadata": {"exitCode": int(sys.argv[2])}
}))' "$LAST_CMD" "$EXIT_CODE" 2>/dev/null)
    elif command -v python >/dev/null 2>&1; then
        PAYLOAD=$(python -c '
import sys, json
print(json.dumps({
    "type": "terminal_command",
    "content": sys.argv[1],
    "metadata": {"exitCode": int(sys.argv[2])}
}))' "$LAST_CMD" "$EXIT_CODE" 2>/dev/null)
    fi

    # Python이 없거나 실행에 실패했을 경우 (fallback 안전 이스케이프)
    if [ -z "$PAYLOAD" ]; then
        # 백슬래시를 먼저 이스케이프한 뒤 따옴표를 이스케이프
        local ESCAPED_CMD=$(echo "$LAST_CMD" | sed 's/\\/\\\\/g; s/"/\\"/g')
        PAYLOAD="{\"type\":\"terminal_command\",\"content\":\"$ESCAPED_CMD\",\"metadata\":{\"exitCode\":$EXIT_CODE}}"
    fi

    # 데몬 API로 전송 (비동기 처리)
    (curl -s -X POST -H "Content-Type: application/json" \
          -d "$PAYLOAD" \
          http://127.0.0.1:${CF_PORT:-49152}/event > /dev/null 2>&1 &)
}

# Bash 연동
if [ -n "$BASH_VERSION" ]; then
    PROMPT_COMMAND="_contextflow_hook; $PROMPT_COMMAND"
# Zsh 연동
elif [ -n "$ZSH_VERSION" ]; then
    precmd_functions+=(_contextflow_hook)
fi
