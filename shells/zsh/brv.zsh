# brv - Fast and safe abbreviation expansion for zsh
# Source this file in your .zshrc:
#   source /path/to/brv.zsh

# Space キーで abbreviation 展開
brv-expand-space() {
  local -a out
  out=( "${(f)$(brv expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )

  case $out[1] in
    success)
      if [[ -n $out[2] ]]; then
        BUFFER=$out[2]
        CURSOR=$out[3]
      else
        zle self-insert
      fi
      ;;
    evaluate)
      # コマンド評価
      local result
      result=$(eval "$out[2]" 2>/dev/null)
      if [[ -n $result ]]; then
        BUFFER="${out[3]}${result}${out[4]}"
        CURSOR=$(( ${#out[3]} + ${#result} ))
      else
        zle self-insert
      fi
      ;;
    stale_cache)
      # キャッシュが古い場合は再コンパイル
      brv compile 2>/dev/null
      # 再試行
      out=( "${(f)$(brv expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )
      if [[ $out[1] == "success" && -n $out[2] ]]; then
        BUFFER=$out[2]
        CURSOR=$out[3]
      else
        zle self-insert
      fi
      ;;
    *)
      zle self-insert
      ;;
  esac
}

# Enter キーで abbreviation 展開 + 実行
brv-expand-accept() {
  local -a out
  out=( "${(f)$(brv expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )

  case $out[1] in
    success)
      if [[ -n $out[2] ]]; then
        BUFFER=$out[2]
      fi
      ;;
    evaluate)
      local result
      result=$(eval "$out[2]" 2>/dev/null)
      if [[ -n $result ]]; then
        BUFFER="${out[3]}${result}${out[4]}"
      fi
      ;;
    stale_cache)
      brv compile 2>/dev/null
      out=( "${(f)$(brv expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )
      if [[ $out[1] == "success" && -n $out[2] ]]; then
        BUFFER=$out[2]
      fi
      ;;
  esac

  zle accept-line
}

# Tab キーでプレースホルダージャンプ
brv-next-placeholder() {
  local -a out
  out=( "${(f)$(brv next-placeholder --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )
  if [[ $out[1] == "success" && -n $out[2] ]]; then
    BUFFER=$out[2]
    CURSOR=$out[3]
  else
    # プレースホルダーがなければ通常の Tab 補完
    zle expand-or-complete
  fi
}

# ウィジェット登録
zle -N brv-expand-space
zle -N brv-expand-accept
zle -N brv-next-placeholder

# キーバインド
bindkey " " brv-expand-space
bindkey "^M" brv-expand-accept
bindkey "^I" brv-next-placeholder
