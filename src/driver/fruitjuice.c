#include <linux/module.h>
#include <linux/power_supply.h>
#include <linux/slab.h>
#include <linux/list.h>
#include <linux/mutex.h>
#include <linux/kernel.h>
#include <linux/device.h>
#include <linux/platform_device.h>

struct idev_inst {
    struct list_head list;
    struct power_supply *psy;
    struct power_supply_desc desc;
    char name[64];
    char model[128];
    char serial[128];
    int capacity;
    int status;
    int present;
    int voltage_now;
    int cycle_count;
    int charge_full_design;
    int charge_full;
    int charge_now;
    struct device_attribute attr_model;
    struct device_attribute attr_serial;
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
    case POWER_SUPPLY_PROP_POWER_NOW:
        val->intval = 0;
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
        return 1;
    default:
        return 0;
    }
}

static ssize_t show_model(struct device *dev, struct device_attribute *attr, char *buf)
{
    struct power_supply *psy = dev_get_drvdata(dev);
    struct idev_inst *inst = power_supply_get_drvdata(psy);

    return sprintf(buf, "%s\n", inst->model);
}

static ssize_t set_model(struct device *dev, struct device_attribute *attr, const char *buf, size_t count)
{
    struct power_supply *psy = dev_get_drvdata(dev);
    struct idev_inst *inst = power_supply_get_drvdata(psy);

    strscpy(inst->model, buf, sizeof(inst->model));
    if (count > 0 && inst->model[strlen(inst->model) - 1] == '\n') {
        inst->model[strlen(inst->model) - 1] = '\0';
    }
    power_supply_changed(psy);
    return count;
}

static ssize_t show_serial(struct device *dev, struct device_attribute *attr, char *buf)
{
    struct power_supply *psy = dev_get_drvdata(dev);
    struct idev_inst *inst = power_supply_get_drvdata(psy);

    return sprintf(buf, "%s\n", inst->serial);
}

static ssize_t set_serial(struct device *dev, struct device_attribute *attr, const char *buf, size_t count)
{
    struct power_supply *psy = dev_get_drvdata(dev);
    struct idev_inst *inst = power_supply_get_drvdata(psy);

    strscpy(inst->serial, buf, sizeof(inst->serial));
    if (count > 0 && inst->serial[strlen(inst->serial) - 1] == '\n') {
        inst->serial[strlen(inst->serial) - 1] = '\0';
    }
    power_supply_changed(psy);
    return count;
}

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
    int ret;

    if (sscanf(buf, "%31s %127s", class_name, udid) < 2) {
        return -EINVAL;
    }

    inst = kzalloc(sizeof(*inst), GFP_KERNEL);
    if (!inst) {
        return -ENOMEM;
    }

    snprintf(inst->name, sizeof(inst->name), "fj_%s_%s", class_name, udid);

    mutex_lock(&idev_lock);
    {
        struct idev_inst *existing;
        list_for_each_entry(existing, &idev_devices, list) {
            if (strcmp(existing->name, inst->name) == 0) {
                mutex_unlock(&idev_lock);
                kfree(inst);
                return count;
            }
        }
    }
    mutex_unlock(&idev_lock);
    
    inst->status = POWER_SUPPLY_STATUS_UNKNOWN;
    inst->present = 0;
    
    inst->desc.name = inst->name;
    inst->desc.type = POWER_SUPPLY_TYPE_BATTERY;
    inst->desc.properties = idev_props;
    inst->desc.num_properties = ARRAY_SIZE(idev_props);
    inst->desc.get_property = idev_get_property;
    inst->desc.set_property = idev_set_property;
    inst->desc.property_is_writeable = idev_property_is_writeable;

    psy_cfg.drv_data = inst;
    inst->psy = power_supply_register(&idev_pdev->dev, &inst->desc, &psy_cfg);
    if (IS_ERR(inst->psy)) {
        ret = PTR_ERR(inst->psy);
        kfree(inst);
        return ret;
    }

    sysfs_attr_init(&inst->attr_model.attr);
    inst->attr_model.attr.name = "model_name_sync";
    inst->attr_model.attr.mode = 0644;
    inst->attr_model.show = show_model;
    inst->attr_model.store = set_model;

    sysfs_attr_init(&inst->attr_serial.attr);
    inst->attr_serial.attr.name = "serial_number_sync";
    inst->attr_serial.attr.mode = 0644;
    inst->attr_serial.show = show_serial;
    inst->attr_serial.store = set_serial;

    ret = device_create_file(&inst->psy->dev, &inst->attr_model);
    if (ret) {
        goto err_psy;
    }
    ret = device_create_file(&inst->psy->dev, &inst->attr_serial);
    if (ret) {
        goto err_attr;
    }

    mutex_lock(&idev_lock);
    list_add(&inst->list, &idev_devices);
    mutex_unlock(&idev_lock);

    return count;

err_attr:
    device_remove_file(&inst->psy->dev, &inst->attr_model);
err_psy:
    power_supply_unregister(inst->psy);
    kfree(inst);
    return ret;
}

static ssize_t remove_device_store(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
{
    char name[64];
    struct idev_inst *inst, *tmp;
    int found = 0;

    if (sscanf(buf, "%63s", name) < 1) {
        return -EINVAL;
    }

    mutex_lock(&idev_lock);
    list_for_each_entry_safe(inst, tmp, &idev_devices, list) {
        if (strcmp(inst->name, name) == 0) {
            list_del(&inst->list);
            found = 1;
            break;
        }
    }

    if (found) {
        device_remove_file(&inst->psy->dev, &inst->attr_serial);
        device_remove_file(&inst->psy->dev, &inst->attr_model);
        power_supply_unregister(inst->psy);
        kfree(inst);
    }
    mutex_unlock(&idev_lock);

    return count;
}

static ssize_t update_device_store(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
{
    char name[64] = {0};
    char status_str[32] = {0};
    char serial[128] = {0};
    char model[128] = {0};
    int cap = -1, present = 1, voltage = 0, cycles = 0;
    int charge_full_design = 0, charge_full = 0, charge_now = 0;
    struct idev_inst *inst, *target = NULL;
    char *orig, *str, *token;

    orig = kstrdup(buf, GFP_KERNEL);
    if (!orig)
        return -ENOMEM;

    str = orig;
    while ((token = strsep(&str, " \t\n")) != NULL) {
        char *eq;
        int i;
        if (!*token)
            continue;
        eq = strchr(token, '=');
        if (!eq)
            continue;
        *eq = '\0';
        eq++;
        if (strcmp(token, "name") == 0) {
            strscpy(name, eq, sizeof(name));
        } else if (strcmp(token, "cap") == 0) {
            if (kstrtoint(eq, 10, &cap)) cap = -1;
        } else if (strcmp(token, "status") == 0) {
            strscpy(status_str, eq, sizeof(status_str));
        } else if (strcmp(token, "present") == 0) {
            if (kstrtoint(eq, 10, &present)) present = 1;
        } else if (strcmp(token, "voltage") == 0) {
            if (kstrtoint(eq, 10, &voltage)) voltage = 0;
        } else if (strcmp(token, "cycles") == 0) {
            if (kstrtoint(eq, 10, &cycles)) cycles = 0;
        } else if (strcmp(token, "full_design") == 0) {
            if (kstrtoint(eq, 10, &charge_full_design)) charge_full_design = 0;
        } else if (strcmp(token, "full") == 0) {
            if (kstrtoint(eq, 10, &charge_full)) charge_full = 0;
        } else if (strcmp(token, "now") == 0) {
            if (kstrtoint(eq, 10, &charge_now)) charge_now = 0;
        } else if (strcmp(token, "serial") == 0) {
            strscpy(serial, eq, sizeof(serial));
        } else if (strcmp(token, "model") == 0) {
            strscpy(model, eq, sizeof(model));
            for (i = 0; model[i]; i++) {
                if (model[i] == '_') model[i] = ' ';
            }
        }
    }
    kfree(orig);

    if (!name[0])
        return -EINVAL;

    mutex_lock(&idev_lock);
    list_for_each_entry(inst, &idev_devices, list) {
        if (strcmp(inst->name, name) == 0) {
            target = inst;
            break;
        }
    }

    if (target) {
        if (cap >= 0) target->capacity = cap;
        target->present = present;
        if (voltage > 0) target->voltage_now = voltage;
        if (cycles >= 0) target->cycle_count = cycles;
        if (charge_full_design > 0) target->charge_full_design = charge_full_design;
        if (charge_full > 0) target->charge_full = charge_full;
        if (charge_now > 0) target->charge_now = charge_now;
        if (serial[0]) strscpy(target->serial, serial, sizeof(target->serial));
        if (model[0]) strscpy(target->model, model, sizeof(target->model));

        if (strcmp(status_str, "Charging") == 0)
            target->status = POWER_SUPPLY_STATUS_CHARGING;
        else if (strcmp(status_str, "Discharging") == 0)
            target->status = POWER_SUPPLY_STATUS_DISCHARGING;
        else if (strcmp(status_str, "Not_charging") == 0 || strcmp(status_str, "Not charging") == 0)
            target->status = POWER_SUPPLY_STATUS_NOT_CHARGING;
        else if (strcmp(status_str, "Full") == 0)
            target->status = POWER_SUPPLY_STATUS_FULL;
        else if (status_str[0])
            target->status = POWER_SUPPLY_STATUS_UNKNOWN;

        power_supply_changed(target->psy);
    }
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

    mutex_lock(&idev_lock);
    list_for_each_entry_safe(inst, tmp, &idev_devices, list) {
        device_remove_file(&inst->psy->dev, &inst->attr_serial);
        device_remove_file(&inst->psy->dev, &inst->attr_model);
        power_supply_unregister(inst->psy);
        list_del(&inst->list);
        kfree(inst);
    }
    mutex_unlock(&idev_lock);
    kobject_put(idev_kobj);
    platform_device_unregister(idev_pdev);
}

module_init(idev_factory_init);
module_exit(idev_factory_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("turannul");
MODULE_DESCRIPTION("FruitJuice: iDevice Battery Bridge");
MODULE_VERSION("1.2");
