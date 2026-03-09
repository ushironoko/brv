# kort - Fast and safe abbreviation expansion for zsh
# Source this file in your .zshrc:
#   source /path/to/kort.zsh

# Expand abbreviation on Space key
kort-expand-space() {
  local -a out
  out=( "${(f)$(kort expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )

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
      # Command evaluation
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
      # Recompile if cache is stale
      kort compile 2>/dev/null
      # Retry
      out=( "${(f)$(kort expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )
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

# Expand abbreviation on Enter key and execute
kort-expand-accept() {
  local -a out
  out=( "${(f)$(kort expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )

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
      kort compile 2>/dev/null
      out=( "${(f)$(kort expand --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )
      if [[ $out[1] == "success" && -n $out[2] ]]; then
        BUFFER=$out[2]
      fi
      ;;
  esac

  zle accept-line
}

# Jump to next placeholder on Tab key
kort-next-placeholder() {
  local -a out
  out=( "${(f)$(kort next-placeholder --lbuffer="$LBUFFER" --rbuffer="$RBUFFER")}" )
  if [[ $out[1] == "success" && -n $out[2] ]]; then
    BUFFER=$out[2]
    CURSOR=$out[3]
  else
    # Fall back to normal tab completion if no placeholder
    zle expand-or-complete
  fi
}

# Register widgets
zle -N kort-expand-space
zle -N kort-expand-accept
zle -N kort-next-placeholder

# Key bindings
bindkey " " kort-expand-space
bindkey "^M" kort-expand-accept
bindkey "^I" kort-next-placeholder
