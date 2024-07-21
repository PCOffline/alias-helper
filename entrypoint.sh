#!/bin/zsh
# # function find_alias() {
#     local temp_file_name="alias_use.temp"
#     #! Remove when moving to Oh My Zsh
#     source ~/.zshrc

#     if [[ ! -f $temp_file_name ]]; then
#       touch $temp_file_name
#     fi

#      if [[ $(cat $temp_file_name) != "$@" ]]; then
#         echo "$@" > $temp_file_name

#         local base_commmand="$1"
#         local best_match="$(alias | grep -Fi "='$@")"

#         if [[ $best_match == "" ]]; then
#             # No match
#         else
#           echo $best_match
#         fi
#     fi
# # }
source ~/.zshrc
alias | cargo run