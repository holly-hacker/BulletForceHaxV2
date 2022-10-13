# This file contains commands for Nushell: https://nushell.sh
# You can get these commands using `source commands.nu`

# Looks through the latest debug log to find a specifix Photon message.
# 
# This is a test
def bfh_find_packet [
        type: string, # The type of the message, eg. "EventData" or "OperationReques".
        code: int     # The event/operation code of this event, eg. 229.
    ] {
    
    let file_name = (ls bfhax_data\logs\ | sort-by modified --reverse | first | get name)
    open --raw $file_name | from json --objects | where $it.fields.message == "Message data" and $it.fields.message_code == $code and $it.fields.message_type == $type | get fields.message_data
}
