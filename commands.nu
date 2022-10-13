# This file contains commands for Nushell: https://nushell.sh
# You can get these commands using `source commands.nu`

# Looks through the latest BulletForceHax debug log to find a specifix Photon message.
# This command depends in the `jq` cli utility being present.
def find_log [
        type: string, # The type of the message, eg. "EventData" or "OperationReques".
        code: int     # The event/operation code of this event, eg. 229.
    ] {
    let latest_log_file = (ls bfhax_data\logs\ | sort-by modified --reverse | first | get name)

    let content = (open --raw $latest_log_file | jq $".fields | select\(.message_code == ($code) and .message_type == \"($type)\"\) | .message_data" | from json --objects)
    $content
}
