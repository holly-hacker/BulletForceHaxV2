# This file contains commands for Nushell: https://nushell.sh
# You can get these commands using `source commands.nu`

# Looks through the latest debug log to find a specifix Photon message.
def bfh_find_packet [
        type: string, # The type of the message, eg. "EventData" or "OperationReques".
        code: int     # The event/operation code of this event, eg. 229.
    ] {
    bfh_latest_log_file | where $it.fields.message == "Message data" and $it.fields.message_code == $code and $it.fields.message_type == $type | get fields.message_data
}

# Gets a list of all RPC calls that occured in in the latest log file
def bfh_rpc_calls [] {
    bfh_latest_log_file | where $it.fields.message == "Incoming RPC call" | get fields | select sender method_name parameters
}

# Opens the latest debug log file as json
def bfh_latest_log_file [] {
    let file_name = (ls bfhax_data\logs\ | sort-by modified --reverse | first | get name)
    open --raw $file_name | from json --objects
}
