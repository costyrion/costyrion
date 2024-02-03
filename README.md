# costyrion

### Resources - domain modeling

    // first class object for grouping costs
    pub struct Resource {}

    // aggregation of similar resources, pools are linked to resource drivers
    pub struct ResourcePool {}

    // factors that cause the consumption of resources
    pub struct ResourceCostDriver {}

    // indicator that represents the amount or volume of work done or services provided by a resource
    pub struct ResourceOutputMeasure {}

    // individual or entity that has responsibility and authority over a specific resource
    pub struct ResourceOwner {}

    // total amount of output or work that a resource can produce or handle within a given period under
    // normal operating conditions
    pub struct ResourceCapacity {}

    // refers to the portion of total capacity that is not being used
    pub struct IdleCapacity {}

    // typically a location, department, or unit within an organization where costs related to specific
    // resources are accumulated
    pub struct ResourceCostCenter {}

    // represents a specific, identifiable cost associated with a resource (raw materials, labor,
    // energy, depreciation of equipment, maintenance costs, etc.)
    pub struct ResourceCostElement {}

    pub enum ResourceType {
        Direct,   // can be directly traced to the production of a specific product or service
        Indirect, // cannot be directly traced to the production of a specific product or service
        Shared,   // indirect resource that are shared among different products services, or
                // departments within an organization
    }

    pub enum ResourceAdaptabilityType {
        Commited, // to which a company has made a long-term commitment
        Flexible, // can be quickly adjusted or scaled in response to changes in business activity
    }

    pub enum ResourceCapacityType {
        Theoretical, // or Mximym - the absolute maximum output a resource can produce under ideal
        // conditions, running at full efficiency without any downtime for maintenance,
        // breaks, or disruptions
        Practical, // or Attainable - takes into account real-world conditions like scheduled
        // maintenance, normal downtime, and human factors
        Normal, // expected under average or typical operating conditions over a period,
        // considers the usual fluctuations in demand and operational efficiency,
        // providing a more long-term view of capacity utilization
        Actual, // achieved during a specific period
    }

    pub enum ResourceCostElementType {
        Fixed,
        Variable,
    }