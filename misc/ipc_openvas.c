#include "ipc_openvas.h"

#include <glib.h> /* for g_error */
#include <json-glib/json-glib.h>
#undef G_LOG_DOMAIN
/**
 * @brief GLib logging domain.
 */
#define G_LOG_DOMAIN "lib  misc"

struct ipc_data *
ipc_data_type_from_hostname (const char *source, size_t source_len,
                             const char *hostname, size_t hostname_len)
{
  struct ipc_data *data = NULL;
  struct ipc_hostname *hnd = NULL;
  if (source == NULL || hostname == NULL)
    return NULL;
  if ((data = calloc (1, sizeof (*data))) == NULL)
    return NULL;
  data->type = IPC_DT_HOSTNAME;
  if ((hnd = calloc (1, sizeof (*hnd))) == NULL)
    goto failure_exit;
  hnd->hostname = g_strdup (hostname);
  hnd->source = g_strdup (source);
  hnd->hostname_len = hostname_len;
  hnd->source_len = source_len;
  data->data = hnd;
  return data;
failure_exit:
  free (data);
  return NULL;
}

void
ipc_hostname_destroy (struct ipc_hostname *data)
{
  if (data == NULL)
    return;
  g_free (data->hostname);
  g_free (data->source);
  g_free (data);
}

void
ipc_data_destroy (struct ipc_data *data)
{
  if (data == NULL)
    return;
  switch (data->type)
    {
    case IPC_DT_HOSTNAME:
      ipc_hostname_destroy (data->data);
      break;
    }
  g_free (data);
}

const char *
ipc_data_to_json (struct ipc_data *data)
{
  JsonBuilder *builder;
  JsonGenerator *gen;
  JsonNode *root;
  gchar *json_str;
  struct ipc_hostname *hn = NULL;
  if (data == NULL)
    return NULL;

  builder = json_builder_new ();

  json_builder_begin_object (builder);

  json_builder_set_member_name (builder, "type");
  builder = json_builder_add_int_value (builder, data->type);
  switch (data->type)
    {
    case IPC_DT_HOSTNAME:
      hn = data->data;
      json_builder_set_member_name (builder, "source");
      builder = json_builder_add_string_value (builder, hn->source);
      json_builder_set_member_name (builder, "hostname");
      builder = json_builder_add_string_value (builder, hn->hostname);

      break;
    }

  json_builder_end_object (builder);

  gen = json_generator_new ();
  root = json_builder_get_root (builder);
  json_generator_set_root (gen, root);
  json_str = json_generator_to_data (gen, NULL);

  json_node_free (root);
  g_object_unref (gen);
  g_object_unref (builder);

  if (json_str == NULL)
    g_warning ("%s: Error while creating JSON.", __func__);

  return json_str;
}

struct ipc_data *
ipc_data_from_json (const char *json, size_t len)
{
  JsonParser *parser;
  JsonReader *reader = NULL;

  GError *err = NULL;
  struct ipc_data *ret = NULL;
  void *data = NULL;
  struct ipc_hostname *hn;

  enum ipc_data_type type = -1;

  parser = json_parser_new ();
  if (!json_parser_load_from_data (parser, json, len, &err))
    {
      goto cleanup;
    }

  reader = json_reader_new (json_parser_get_root (parser));

  if (!json_reader_read_member (reader, "type"))
    {
      goto cleanup;
    }
  type = json_reader_get_int_value (reader);
  json_reader_end_member (reader);
  switch (type)
    {
    case IPC_DT_HOSTNAME:
      if ((hn = calloc (1, sizeof (*hn))) == NULL)
        goto cleanup;
      if (!json_reader_read_member (reader, "hostname"))
        {
          goto cleanup;
        }
      hn->hostname = g_strdup (json_reader_get_string_value (reader));
      hn->hostname_len = strlen (hn->hostname);
      json_reader_end_member (reader);
      if (!json_reader_read_member (reader, "source"))
        {
          goto cleanup;
        }
      hn->source = g_strdup (json_reader_get_string_value (reader));
      hn->source_len = strlen (hn->source);
      json_reader_end_member (reader);
      data = hn;
      break;
    }
  if ((ret = calloc (1, sizeof (*ret))) == NULL)
    goto cleanup;
  ret->type = type;
  ret->data = data;
cleanup:
  if (reader)
    g_object_unref (reader);
  g_object_unref (parser);
  if (err != NULL)
    {
      g_warning ("%s: Unable to parse json (%s). Reason: %s", __func__, json,
                 err->message);
    }
  if (ret == NULL)
    {
      if (data != NULL)
        {
          switch (type)
            {
            case IPC_DT_HOSTNAME:
              ipc_hostname_destroy (data);
              break;
            }
        }
    }
  return ret;
}
