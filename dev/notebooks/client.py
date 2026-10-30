import marimo

__generated_with = "0.14.17"
app = marimo.App()


@app.cell
def _():
    import os
    from pprint import pprint

    from unitycatalog_client import UnityCatalogClient

    return UnityCatalogClient, os, pprint


@app.cell
def _(mo):
    mo.md(
        r"""
    ## Unity Catalog Client

    This notebook demonstrates how to interact with unity catalog APIs using the
    `unitycatalog-rs` python bindings. We will walk through some common scenarios
    when interacting with UC.

    * Setup client
    * Create Catalog/Schema/Table
    * Read from and write to table
    * Create and query a Share
    """
    )
    return


@app.cell
def _(mo):
    mo.md(r"""### Setup unity client""")
    return


@app.cell
def _(UnityCatalogClient, os):
    # connect to databricks hosted service using a PAT
    host = os.environ["DATABRICKS_HOST"]
    client = UnityCatalogClient(
        base_url=f"{host}/api/2.1/unity-catalog/",
        token=os.environ["DATABRICKS_TOKEN"],
    )

    # connect to locally running service
    # client = UnityCatalogClient(
    #     base_url="http://localhost:8080/api/2.1/unity-catalog/"
    # )
    return (client,)


@app.cell
def _(mo):
    mo.md(
        r"""
    ### Manage Catalogs, Schemas, and Tables

    We'll create and then cleanup a catalog and child schemas. Then register
    a table with the catalog.
    """
    )
    return


@app.cell
def _(client):
    client.list_catalogs()
    return


@app.cell
def _(client):
    client.catalog(name="ext_client").schema(name="simple").table(name="super_simple").get(
        False, False, False
    )
    return


@app.cell
def _(client):
    client.temporary_credentials().temporary_table_credential(
        "ext_client.simple.super_simple", "READ"
    )
    return


@app.cell
def _(client, pprint):
    new_catalog = client.create_catalog(name="new_catalog")
    print("Created new catalog:")
    pprint(new_catalog)
    return


@app.cell
def _(client):
    client.catalog(name="test_catalog").create_schema(name="test_schema")
    return


@app.cell
def _(client, pprint):
    # create a new catalog at the root.
    new_catalog = client.create_catalog(name="new_catalog")
    print("Created new catalog:")
    pprint(new_catalog)

    # get an instance client for the catalog.
    catalog_client = client.catalog(name=new_catalog.name)

    # update the catalog.
    updated_catalog = catalog_client.update(comment="new comment")
    print("Updated catalog:")
    pprint(updated_catalog)

    # create a schema
    schema_info = catalog_client.create_schema("new_schema")
    pprint("created schema")
    pprint(schema_info)

    schema_client = catalog_client.schema(schema_info.name)

    updated_schema = schema_client.update(comment="schema comment")
    print("Updated schema:")
    pprint(updated_schema)

    schema_client.delete()
    print("Deleted schema.")

    catalog_client.delete()
    print("Deleted catalog.")
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(
        r"""
    ### Delta Sharing

    Set up a share and access it via delta sharing.
    """
    )
    return


@app.cell
def _(client):
    shared_catalog = client.create_catalog(name="shared_catalog")
    shared_catalog_client = client.catalog(name=shared_catalog.name)
    _shared_schema = shared_catalog_client.create_schema(schema_name="shared_schema")

    # TODO: add tables
    return


@app.cell
def _(client):
    share_info = client.create_share(name="new_share")
    share_client = client.share(name=share_info.name)
    return (share_client,)


@app.cell
def _(share_client):
    from unitycatalog_client import (
        Action,
        DataObject,
        DataObjectType,
        DataObjectUpdate,
        HistoryStatus,
    )

    share = {
        "updates": [
            DataObjectUpdate(
                **{
                    "action": Action.Add,
                    "data_object": DataObject(
                        **{
                            "name": "dat.dat.all_primitive_types",
                            "data_object_type": DataObjectType.Table,
                            "shared_as": "dat.all_primitive_types",
                            "partitions": [],
                            "history_data_sharing_status": HistoryStatus.Disabled,
                            "enable_cdf": False,
                        }
                    ),
                }
            ),
            DataObjectUpdate(
                **{
                    "action": Action.Add,
                    "data_object": DataObject(
                        **{
                            "name": "dat.dat.column_mapping",
                            "data_object_type": DataObjectType.Table,
                            "shared_as": "dat.column_mapping",
                            "partitions": [],
                            "history_data_sharing_status": HistoryStatus.Disabled,
                            "enable_cdf": False,
                        }
                    ),
                }
            ),
        ]
    }

    share_client.update(**share)
    return


@app.cell
def _():
    import marimo as mo

    return (mo,)


if __name__ == "__main__":
    app.run()
