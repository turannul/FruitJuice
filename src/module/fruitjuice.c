#include <linux/module.h>
#include <linux/power_supply.h>
#include <linux/slab.h>
#include <linux/list.h>
#include <linux/mutex.h>
#include <linux/kernel.h>
#include <linux/device.h>
#include <linux/platform_device.h>
#include <linux/ctype.h>
#include <linux/string.h>

struct idev_inst {
    struct list_head list;
    struct power_supply *psy;
    struct power_supply_desc desc;
    char name[128];
    char udid[128];
    char model[128];
    char device_model[128];
    char model_detail[128];
    char serial[128];
    char os_version[32];
    char product_type[64];
    char hardware_model[64];
    int capacity;
    int status;
    int present;
    int voltage_now;
    int cycle_count;
    int charge_full_design;
    int charge_full;
    int charge_now;
    int current_now;
    int power_now;
    int time_to_empty_now;
    int time_to_full_now;
    int capacity_level;
    int health;
    int health_percent;
    int raw_capacity;
    int adapter_watts;
    int adapter_voltage;
    int is_wireless;
};

static LIST_HEAD(idev_devices);
static DEFINE_MUTEX(idev_lock);
static struct platform_device *idev_pdev;

static int idev_get_property(struct power_supply *psy,
                               enum power_supply_property psp,
                               union power_supply_propval *val)
{
    struct idev_inst *inst = power_supply_get_drvdata(psy);

    switch (psp) {
    case POWER_SUPPLY_PROP_STATUS:
        val->intval = inst->status;
        break;
    case POWER_SUPPLY_PROP_CAPACITY:
        val->intval = inst->capacity;
        break;
    case POWER_SUPPLY_PROP_PRESENT:
        val->intval = inst->present;
        break;
    case POWER_SUPPLY_PROP_VOLTAGE_NOW:
        val->intval = inst->voltage_now;
        break;
    case POWER_SUPPLY_PROP_CYCLE_COUNT:
        val->intval = inst->cycle_count;
        break;
    case POWER_SUPPLY_PROP_CHARGE_FULL_DESIGN:
        val->intval = inst->charge_full_design;
        break;
    case POWER_SUPPLY_PROP_CHARGE_FULL:
        val->intval = inst->charge_full;
        break;
    case POWER_SUPPLY_PROP_CHARGE_NOW:
        val->intval = inst->charge_now;
        break;
    case POWER_SUPPLY_PROP_CURRENT_NOW:
        val->intval = inst->current_now;
        break;
    case POWER_SUPPLY_PROP_POWER_NOW:
        if (inst->power_now > 0)
            val->intval = inst->power_now;
        else if (inst->voltage_now > 0 && inst->current_now != 0)
            val->intval = (int)(abs((long long)inst->voltage_now * inst->current_now / 1000000LL));
        else
            val->intval = 0;
        break;
    case POWER_SUPPLY_PROP_TIME_TO_EMPTY_NOW:
    case POWER_SUPPLY_PROP_TIME_TO_EMPTY_AVG:
        val->intval = inst->time_to_empty_now;
        break;
    case POWER_SUPPLY_PROP_TIME_TO_FULL_NOW:
    case POWER_SUPPLY_PROP_TIME_TO_FULL_AVG:
        val->intval = inst->time_to_full_now;
        break;
    case POWER_SUPPLY_PROP_CAPACITY_LEVEL:
        val->intval = inst->capacity_level;
        break;
    case POWER_SUPPLY_PROP_HEALTH:
        val->intval = inst->health;
        break;
    case POWER_SUPPLY_PROP_SCOPE:
        val->intval = POWER_SUPPLY_SCOPE_DEVICE;
        break;
    case POWER_SUPPLY_PROP_MANUFACTURER:
        val->strval = "Apple";
        break;
    case POWER_SUPPLY_PROP_TECHNOLOGY:
        val->intval = POWER_SUPPLY_TECHNOLOGY_LION;
        break;
    case POWER_SUPPLY_PROP_MODEL_NAME:
        val->strval = inst->model;
        break;
    case POWER_SUPPLY_PROP_SERIAL_NUMBER:
        val->strval = inst->serial;
        break;
    default:
        return -EINVAL;
    }
    return 0;
}

static int idev_set_property(struct power_supply *psy,
                               enum power_supply_property psp,
                               const union power_supply_propval *val)
{
    struct idev_inst *inst = power_supply_get_drvdata(psy);

    switch (psp) {
    case POWER_SUPPLY_PROP_CAPACITY:
        inst->capacity = val->intval;
        break;
    case POWER_SUPPLY_PROP_STATUS:
        inst->status = val->intval;
        break;
    case POWER_SUPPLY_PROP_PRESENT:
        inst->present = val->intval;
        break;
    case POWER_SUPPLY_PROP_VOLTAGE_NOW:
        inst->voltage_now = val->intval;
        break;
    case POWER_SUPPLY_PROP_CYCLE_COUNT:
        inst->cycle_count = val->intval;
        break;
    case POWER_SUPPLY_PROP_CHARGE_FULL_DESIGN:
        inst->charge_full_design = val->intval;
        break;
    case POWER_SUPPLY_PROP_CHARGE_FULL:
        inst->charge_full = val->intval;
        break;
    case POWER_SUPPLY_PROP_CHARGE_NOW:
        inst->charge_now = val->intval;
        break;
    case POWER_SUPPLY_PROP_CURRENT_NOW:
        inst->current_now = val->intval;
        break;
    case POWER_SUPPLY_PROP_POWER_NOW:
        inst->power_now = val->intval;
        break;
    case POWER_SUPPLY_PROP_TIME_TO_EMPTY_NOW:
        inst->time_to_empty_now = val->intval;
        break;
    case POWER_SUPPLY_PROP_TIME_TO_FULL_NOW:
        inst->time_to_full_now = val->intval;
        break;
    default:
        return -EINVAL;
    }
    power_supply_changed(psy);
    return 0;
}

static int idev_property_is_writeable(struct power_supply *psy,
                                        enum power_supply_property psp)
{
    switch (psp) {
    case POWER_SUPPLY_PROP_CAPACITY:
    case POWER_SUPPLY_PROP_STATUS:
    case POWER_SUPPLY_PROP_PRESENT:
    case POWER_SUPPLY_PROP_VOLTAGE_NOW:
    case POWER_SUPPLY_PROP_CYCLE_COUNT:
    case POWER_SUPPLY_PROP_CHARGE_FULL_DESIGN:
    case POWER_SUPPLY_PROP_CHARGE_FULL:
    case POWER_SUPPLY_PROP_CHARGE_NOW:
    case POWER_SUPPLY_PROP_CURRENT_NOW:
    case POWER_SUPPLY_PROP_POWER_NOW:
    case POWER_SUPPLY_PROP_TIME_TO_EMPTY_NOW:
    case POWER_SUPPLY_PROP_TIME_TO_FULL_NOW:
        return 1;
    default:
        return 0;
    }
}

/* Helper macros for concise, idiomatic sysfs attribute declarations */
#define IDEV_ATTR_RO_STR(attr_name, member) \
static ssize_t attr_name##_show(struct device *dev, struct device_attribute *attr, char *buf) \
{ \
    struct power_supply *psy = dev_get_drvdata(dev); \
    struct idev_inst *inst = power_supply_get_drvdata(psy); \
    return sysfs_emit(buf, "%s\n", inst->member); \
} \
static DEVICE_ATTR_RO(attr_name)

#define IDEV_ATTR_RO_INT(attr_name, member) \
static ssize_t attr_name##_show(struct device *dev, struct device_attribute *attr, char *buf) \
{ \
    struct power_supply *psy = dev_get_drvdata(dev); \
    struct idev_inst *inst = power_supply_get_drvdata(psy); \
    return sysfs_emit(buf, "%d\n", inst->member); \
} \
static DEVICE_ATTR_RO(attr_name)

/* Declare custom device attributes */
IDEV_ATTR_RO_STR(udid, udid);
IDEV_ATTR_RO_STR(device_model, device_model);
IDEV_ATTR_RO_STR(model, device_model);

static ssize_t model_detail_show(struct device *dev, struct device_attribute *attr, char *buf)
{
    struct power_supply *psy = dev_get_drvdata(dev);
    struct idev_inst *inst = power_supply_get_drvdata(psy);
    return sysfs_emit(buf, "%s\n", (inst->model_detail[0] && strcmp(inst->model_detail, "N/A") != 0) ? inst->model_detail : "N/A");
}
static DEVICE_ATTR_RO(model_detail);

IDEV_ATTR_RO_STR(os_version, os_version);
IDEV_ATTR_RO_STR(product_type, product_type);
IDEV_ATTR_RO_STR(hardware_model, hardware_model);
IDEV_ATTR_RO_INT(health_percent, health_percent);
IDEV_ATTR_RO_INT(raw_capacity, raw_capacity);
IDEV_ATTR_RO_INT(adapter_watts, adapter_watts);
IDEV_ATTR_RO_INT(adapter_voltage, adapter_voltage);
IDEV_ATTR_RO_INT(is_wireless, is_wireless);

static struct attribute *idev_attrs[] = {
    &dev_attr_udid.attr,
    &dev_attr_device_model.attr,
    &dev_attr_model.attr,
    &dev_attr_model_detail.attr,
    &dev_attr_os_version.attr,
    &dev_attr_product_type.attr,
    &dev_attr_hardware_model.attr,
    &dev_attr_health_percent.attr,
    &dev_attr_raw_capacity.attr,
    &dev_attr_adapter_watts.attr,
    &dev_attr_adapter_voltage.attr,
    &dev_attr_is_wireless.attr,
    NULL,
};
ATTRIBUTE_GROUPS(idev);

static enum power_supply_property idev_props[] = {
    POWER_SUPPLY_PROP_STATUS,
    POWER_SUPPLY_PROP_CAPACITY,
    POWER_SUPPLY_PROP_PRESENT,
    POWER_SUPPLY_PROP_VOLTAGE_NOW,
    POWER_SUPPLY_PROP_CYCLE_COUNT,
    POWER_SUPPLY_PROP_CHARGE_FULL_DESIGN,
    POWER_SUPPLY_PROP_CHARGE_FULL,
    POWER_SUPPLY_PROP_CHARGE_NOW,
    POWER_SUPPLY_PROP_CURRENT_NOW,
    POWER_SUPPLY_PROP_POWER_NOW,
    POWER_SUPPLY_PROP_TIME_TO_EMPTY_NOW,
    POWER_SUPPLY_PROP_TIME_TO_EMPTY_AVG,
    POWER_SUPPLY_PROP_TIME_TO_FULL_NOW,
    POWER_SUPPLY_PROP_TIME_TO_FULL_AVG,
    POWER_SUPPLY_PROP_CAPACITY_LEVEL,
    POWER_SUPPLY_PROP_HEALTH,
    POWER_SUPPLY_PROP_SCOPE,
    POWER_SUPPLY_PROP_MODEL_NAME,
    POWER_SUPPLY_PROP_MANUFACTURER,
    POWER_SUPPLY_PROP_TECHNOLOGY,
    POWER_SUPPLY_PROP_SERIAL_NUMBER,
};

static ssize_t add_device_store(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
{
    char class_name[32], udid[128];
    struct idev_inst *inst;
    struct power_supply_config psy_cfg = {};
    int ret, i;

    if (sscanf(buf, "%31s %127s", class_name, udid) < 2) {
        return -EINVAL;
    }

    for (i = 0; class_name[i]; i++) {
        if (!isalnum(class_name[i]) && class_name[i] != '_')
            return -EINVAL;
    }
    for (i = 0; udid[i]; i++) {
        if (!isalnum(udid[i]) && udid[i] != '-' && udid[i] != '_')
            return -EINVAL;
    }

    inst = kzalloc(sizeof(*inst), GFP_KERNEL);
    if (!inst) {
        return -ENOMEM;
    }

    snprintf(inst->name, sizeof(inst->name), "fj_%s_%s", class_name, udid);
    strscpy(inst->udid, udid, sizeof(inst->udid));

    mutex_lock(&idev_lock);
    {
        struct idev_inst *existing;
        list_for_each_entry(existing, &idev_devices, list) {
            if (strcmp(existing->name, inst->name) == 0 || strcmp(existing->udid, inst->udid) == 0) {
                mutex_unlock(&idev_lock);
                kfree(inst);
                return count;
            }
        }
    }
    mutex_unlock(&idev_lock);
    
    inst->status = POWER_SUPPLY_STATUS_UNKNOWN;
    inst->present = 0;
    inst->capacity_level = POWER_SUPPLY_CAPACITY_LEVEL_UNKNOWN;
    inst->health = POWER_SUPPLY_HEALTH_GOOD;
    
    inst->desc.name = inst->name;
    inst->desc.type = POWER_SUPPLY_TYPE_BATTERY;
    inst->desc.properties = idev_props;
    inst->desc.num_properties = ARRAY_SIZE(idev_props);
    inst->desc.get_property = idev_get_property;
    inst->desc.set_property = idev_set_property;
    inst->desc.property_is_writeable = idev_property_is_writeable;

    psy_cfg.drv_data = inst;
    psy_cfg.attr_grp = idev_groups;
    inst->psy = power_supply_register(&idev_pdev->dev, &inst->desc, &psy_cfg);
    if (IS_ERR(inst->psy)) {
        ret = PTR_ERR(inst->psy);
        kfree(inst);
        return ret;
    }

    mutex_lock(&idev_lock);
    list_add(&inst->list, &idev_devices);
    mutex_unlock(&idev_lock);

    return count;
}

static ssize_t remove_device_store(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
{
    char id[128];
    struct idev_inst *inst, *tmp;
    int found = 0;

    if (sscanf(buf, "%127s", id) < 1) {
        return -EINVAL;
    }

    mutex_lock(&idev_lock);
    list_for_each_entry_safe(inst, tmp, &idev_devices, list) {
        if (strcmp(inst->name, id) == 0 || strcmp(inst->udid, id) == 0) {
            list_del(&inst->list);
            found = 1;
            break;
        }
    }
    mutex_unlock(&idev_lock);

    if (found) {
        power_supply_unregister(inst->psy);
        kfree(inst);
    }

    return count;
}

static inline int parse_int_val(const char *v, size_t vlen, int *out)
{
    char buf[32];
    size_t len = min(vlen, sizeof(buf) - 1);
    memcpy(buf, v, len);
    buf[len] = '\0';
    return kstrtoint(buf, 10, out);
}

static inline void parse_str_val(char *dst, size_t dst_sz, const char *v, size_t vlen)
{
    size_t len = min(vlen, dst_sz - 1);
    memcpy(dst, v, len);
    dst[len] = '\0';
}

static inline int parse_battery_status(const char *s)
{
    switch (s[0]) {
    case 'C': return POWER_SUPPLY_STATUS_CHARGING;
    case 'D': return POWER_SUPPLY_STATUS_DISCHARGING;
    case 'N': return POWER_SUPPLY_STATUS_NOT_CHARGING;
    case 'F': return POWER_SUPPLY_STATUS_FULL;
    default:  return s[0] ? POWER_SUPPLY_STATUS_UNKNOWN : -1;
    }
}

static inline int calc_capacity_level(int capacity, int status)
{
    if (capacity <= 5)
        return POWER_SUPPLY_CAPACITY_LEVEL_CRITICAL;
    if (capacity <= 20)
        return POWER_SUPPLY_CAPACITY_LEVEL_LOW;
    if (capacity >= 100 || status == POWER_SUPPLY_STATUS_FULL)
        return POWER_SUPPLY_CAPACITY_LEVEL_FULL;
    if (capacity >= 95)
        return POWER_SUPPLY_CAPACITY_LEVEL_HIGH;
    return POWER_SUPPLY_CAPACITY_LEVEL_NORMAL;
}

#define CASE_STR(key_lit, dest) \
    if (strncmp(k_start, key_lit, sizeof(key_lit) - 1) == 0) { \
        parse_str_val(dest, sizeof(dest), v_start, vlen); \
        break; \
    }

#define CASE_INT(key_lit, dest) \
    if (strncmp(k_start, key_lit, sizeof(key_lit) - 1) == 0) { \
        if (parse_int_val(v_start, vlen, &num_val) == 0) \
            dest = num_val; \
        break; \
    }

static ssize_t update_device_store(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
{
    const char *p = buf;
    const char *end = buf + count;
    char id[128] = {0};
    char status_str[32] = {0};
    struct idev_inst *inst, *target = NULL;

    /* First pass: extract device name or udid to locate target instance under lock */
    while (p < end) {
        const char *k_start, *k_end, *v_start, *v_end;
        size_t klen, vlen;

        while (p < end && (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r'))
            p++;
        if (p >= end)
            break;

        k_start = p;
        while (p < end && *p != '=' && *p != ' ' && *p != '\t' && *p != '\n' && *p != '\r')
            p++;
        k_end = p;

        if (p >= end || *p != '=')
            continue;
        p++; /* skip '=' */

        if (p < end && *p == '"') {
            p++; /* skip opening quote */
            v_start = p;
            while (p < end && *p != '"' && *p != '\n' && *p != '\r')
                p++;
            v_end = p;
            if (p < end && *p == '"')
                p++; /* skip closing quote */
        } else {
            v_start = p;
            while (p < end && *p != ' ' && *p != '\t' && *p != '\n' && *p != '\r')
                p++;
            v_end = p;
        }

        klen = k_end - k_start;
        vlen = v_end - v_start;

        if ((klen == 4 && strncmp(k_start, "name", 4) == 0) ||
            (klen == 4 && strncmp(k_start, "udid", 4) == 0)) {
            parse_str_val(id, sizeof(id), v_start, vlen);
            break;
        }
    }

    if (!id[0])
        return -EINVAL;

    mutex_lock(&idev_lock);
    list_for_each_entry(inst, &idev_devices, list) {
        if (strcmp(inst->name, id) == 0 || strcmp(inst->udid, id) == 0) {
            target = inst;
            break;
        }
    }

    if (!target) {
        mutex_unlock(&idev_lock);
        return count;
    }

    /* Second pass: parse fields directly into target (zero-allocation, O(1) jump table) */
    p = buf;
    while (p < end) {
        const char *k_start, *k_end, *v_start, *v_end;
        size_t klen, vlen;
        int num_val = 0;

        while (p < end && (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r'))
            p++;
        if (p >= end)
            break;

        k_start = p;
        while (p < end && *p != '=' && *p != ' ' && *p != '\t' && *p != '\n' && *p != '\r')
            p++;
        k_end = p;

        if (p >= end || *p != '=')
            continue;
        p++; /* skip '=' */

        if (p < end && *p == '"') {
            p++; /* skip opening quote */
            v_start = p;
            while (p < end && *p != '"' && *p != '\n' && *p != '\r')
                p++;
            v_end = p;
            if (p < end && *p == '"')
                p++; /* skip closing quote */
        } else {
            v_start = p;
            while (p < end && *p != ' ' && *p != '\t' && *p != '\n' && *p != '\r')
                p++;
            v_end = p;
        }

        klen = k_end - k_start;
        vlen = v_end - v_start;

        if (klen == 4 && strncmp(k_start, "name", 4) == 0) {
            continue;
        }

        switch (klen) {
        case 3:
            CASE_INT("cap", target->capacity);
            CASE_INT("now", target->charge_now);
            break;
        case 4:
            CASE_INT("full", target->charge_full);
            CASE_STR("udid", target->udid);
            break;
        case 5:
            CASE_STR("model", target->model);
            CASE_INT("power", target->power_now);
            CASE_INT("watts", target->adapter_watts);
            break;
        case 6:
            CASE_STR("status", status_str);
            CASE_INT("cycles", target->cycle_count);
            CASE_STR("serial", target->serial);
            CASE_STR("os_ver", target->os_version);
            break;
        case 7:
            CASE_INT("present", target->present);
            CASE_INT("voltage", target->voltage_now);
            CASE_INT("current", target->current_now);
            CASE_INT("raw_cap", target->raw_capacity);
            break;
        case 8:
            CASE_INT("wireless", target->is_wireless);
            CASE_STR("hw_model", target->hardware_model);
            break;
        case 9:
            CASE_INT("time_full", target->time_to_full_now);
            CASE_INT("adapter_v", target->adapter_voltage);
            CASE_STR("prod_type", target->product_type);
            break;
        case 10:
            CASE_INT("time_empty", target->time_to_empty_now);
            CASE_INT("health_pct", target->health_percent);
            break;
        case 11:
            CASE_INT("full_design", target->charge_full_design);
            break;
        case 12:
            CASE_STR("device_model", target->device_model);
            CASE_STR("model_detail", target->model_detail);
            break;
        }
    }

    {
        int new_status = parse_battery_status(status_str);
        if (new_status >= 0)
            target->status = new_status;
    }

    target->capacity_level = calc_capacity_level(target->capacity, target->status);
    target->health = (target->capacity == 0 && target->status == POWER_SUPPLY_STATUS_DISCHARGING) ?
        POWER_SUPPLY_HEALTH_DEAD : POWER_SUPPLY_HEALTH_GOOD;

    power_supply_changed(target->psy);
    mutex_unlock(&idev_lock);

    return count;
}

static struct kobj_attribute add_device_attr = __ATTR(add_device, 0200, NULL, add_device_store);
static struct kobj_attribute remove_device_attr = __ATTR(remove_device, 0200, NULL, remove_device_store);
static struct kobj_attribute update_device_attr = __ATTR(update_device, 0200, NULL, update_device_store);
static struct kobject *idev_kobj;

static int __init idev_factory_init(void)
{
    int ret;

    idev_pdev = platform_device_register_simple("fruitjuice", -1, NULL, 0);
    if (IS_ERR(idev_pdev)) {
        return PTR_ERR(idev_pdev);
    }

    idev_kobj = kobject_create_and_add("fruitjuice", kernel_kobj);
    if (!idev_kobj) {
        ret = -ENOMEM;
        goto err_pdev;
    }

    add_device_attr.attr.mode = 0666;
    remove_device_attr.attr.mode = 0666;
    update_device_attr.attr.mode = 0666;

    ret = sysfs_create_file(idev_kobj, &add_device_attr.attr);
    if (ret) goto err_kobj;
    ret = sysfs_create_file(idev_kobj, &remove_device_attr.attr);
    if (ret) goto err_kobj;
    ret = sysfs_create_file(idev_kobj, &update_device_attr.attr);
    if (ret) goto err_kobj;
    return 0;

err_kobj:
    kobject_put(idev_kobj);
err_pdev:
    platform_device_unregister(idev_pdev);
    return ret;
}

static void __exit idev_factory_exit(void)
{
    struct idev_inst *inst, *tmp;
    LIST_HEAD(local_cleanup);

    mutex_lock(&idev_lock);
    list_splice_init(&idev_devices, &local_cleanup);
    mutex_unlock(&idev_lock);

    list_for_each_entry_safe(inst, tmp, &local_cleanup, list) {
        power_supply_unregister(inst->psy);
        list_del(&inst->list);
        kfree(inst);
    }
    kobject_put(idev_kobj);
    platform_device_unregister(idev_pdev);
}

module_init(idev_factory_init);
module_exit(idev_factory_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("turannul");
MODULE_DESCRIPTION("FruitJuice: iDevice Battery Bridge");
MODULE_VERSION("2.0");
