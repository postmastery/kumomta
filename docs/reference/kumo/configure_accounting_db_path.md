# `kumo.configure_accounting_db_path("PATH")`

{{since('dev')}}

Configures the path that will be used for the accounting database.

The accounting database records the total volume of message receptions
and deliveries performed by the MTA.

This function should be called only from inside your [init](../events/init.md)
event handler.

The default path is `"/var/spool/kumomta/accounting.db"`.
