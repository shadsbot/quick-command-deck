/* Automatically generated nanopb header */
/* Generated by nanopb-0.4.5 */

#ifndef PB_TEST_COMMUNIQUE_PB_H_INCLUDED
#define PB_TEST_COMMUNIQUE_PB_H_INCLUDED
#include "pb.h"

#if PB_PROTO_HEADER_VERSION != 40
#error Regenerate this file with the current version of nanopb generator.
#endif

/* Struct definitions */
/* Sent from the arduino firmware to the recieving software */
typedef struct _ButtonPushed { 
    int32_t number; /* corresponds to a command in the software */
} ButtonPushed;

/* Sent from recieving software to arduino firmware as an acknowledgement */
typedef struct _DisplayText { 
    pb_callback_t line1; /* LINE 1 TEXT HERE */
    pb_callback_t line2; /* LINE 2 TEXT HERE */
    int32_t brightness; /* brightness of 1602A backlight. Not sure if this is viable yet. */
    int32_t duration_ms; /* milliseconds to display message */
    bool flash_led; /* may or may not have an LED */
} DisplayText;


#ifdef __cplusplus
extern "C" {
#endif

/* Initializer values for message structs */
#define ButtonPushed_init_default                {0}
#define DisplayText_init_default                 {{{NULL}, NULL}, {{NULL}, NULL}, 0, 0, 0}
#define ButtonPushed_init_zero                   {0}
#define DisplayText_init_zero                    {{{NULL}, NULL}, {{NULL}, NULL}, 0, 0, 0}

/* Field tags (for use in manual encoding/decoding) */
#define ButtonPushed_number_tag                  1
#define DisplayText_line1_tag                    1
#define DisplayText_line2_tag                    2
#define DisplayText_brightness_tag               3
#define DisplayText_duration_ms_tag              4
#define DisplayText_flash_led_tag                5

/* Struct field encoding specification for nanopb */
#define ButtonPushed_FIELDLIST(X, a) \
X(a, STATIC,   SINGULAR, INT32,    number,            1)
#define ButtonPushed_CALLBACK NULL
#define ButtonPushed_DEFAULT NULL

#define DisplayText_FIELDLIST(X, a) \
X(a, CALLBACK, SINGULAR, STRING,   line1,             1) \
X(a, CALLBACK, SINGULAR, STRING,   line2,             2) \
X(a, STATIC,   SINGULAR, INT32,    brightness,        3) \
X(a, STATIC,   SINGULAR, INT32,    duration_ms,       4) \
X(a, STATIC,   SINGULAR, BOOL,     flash_led,         5)
#define DisplayText_CALLBACK pb_default_field_callback
#define DisplayText_DEFAULT NULL

extern const pb_msgdesc_t ButtonPushed_msg;
extern const pb_msgdesc_t DisplayText_msg;

/* Defines for backwards compatibility with code written before nanopb-0.4.0 */
#define ButtonPushed_fields &ButtonPushed_msg
#define DisplayText_fields &DisplayText_msg

/* Maximum encoded size of messages (where known) */
/* DisplayText_size depends on runtime parameters */
#define ButtonPushed_size                        11

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif
