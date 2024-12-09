use crate::prelude::*;
use anyhow::Result;

/// Trait for storing and retrieving json-like documents,
/// implemented by [MongoDb] and [MemoryDb]
#[async_trait::async_trait]
pub trait DocumentDb: 'static + Send + Sync {
	fn scenes(&self) -> &dyn DocumentCollection<SceneDoc>;
	fn crates(&self) -> &dyn DocumentCollection<CrateDoc>;
	async fn clear(&self) -> Result<()> {
		self.scenes().clear().await?;
		self.crates().clear().await?;
		Ok(())
	}
}

#[derive(Clone)]
pub enum DocumentDbEnum {
	Mongo(MongoDb),
	Memory(MemoryDb),
}

impl DocumentDbEnum {
	pub async fn new(env: ApiEnvironment) -> Result<Self> {
		match env {
			ApiEnvironment::Local => Ok(Self::Memory(MemoryDb::new())),
			ApiEnvironment::Staging => {
				Ok(Self::Mongo(MongoDb::new(env).await?))
			}
			ApiEnvironment::Prod => Ok(Self::Mongo(MongoDb::new(env).await?)),
		}
	}


	pub fn inner(&self) -> &dyn DocumentDb {
		// LessE
		match self {
			DocumentDbEnum::Mongo(val) => val,
			DocumentDbEnum::Memory(val) => val,
		}
	}
}



/// Comparison operators return data based on value comparisons.
pub enum ComparisonOperator {
	///Matches values that are equal to a specified value.
	EqualTo,
	///Matches values that are greater than a specified value.
	GreaterThan,
	///Matches values that are greater than or equal to a specified value.
	GreaterThanOrEqualTo,
	///Matches values that are less than a specified value.
	LessThan,
	///Matches values that are less than or equal to a specified value.
	LessThanOrEqualTo,
	///Matches all values that are not equal to a specified value.
	NotEqual,
	///Matches any of the values specified in an array.
	InArray,
	///Matches none of the values specified in an array.
	NotInArray,
	///Matches values that exist, including values that are null.
	Exists,
	///Matches values of a specific type, see https://www.mongodb.com/docs/manual/reference/operator/query/type/#available-types
	Type,
}



impl ComparisonOperator {
	/// https://www.mongodb.com/docs/manual/reference/operator/query-comparison/
	pub fn to_mongo_operator(&self) -> &'static str {
		match self {
			ComparisonOperator::EqualTo => "$eq",
			ComparisonOperator::GreaterThan => "$gt",
			ComparisonOperator::GreaterThanOrEqualTo => "$gte",
			ComparisonOperator::LessThan => "$lt",
			ComparisonOperator::LessThanOrEqualTo => "$lte",
			ComparisonOperator::NotEqual => "$ne",
			ComparisonOperator::InArray => "$in",
			ComparisonOperator::NotInArray => "$nin",
			ComparisonOperator::Exists => "$exists",
			ComparisonOperator::Type => "$type",
		}
	}

	/// comparison operators, in/not in array not implemented
	pub fn compare<T: PartialEq + PartialOrd>(&self, lhs: &T, rhs: &T) -> bool {
		match self {
			ComparisonOperator::EqualTo => lhs == rhs,
			ComparisonOperator::GreaterThan => lhs > rhs,
			ComparisonOperator::GreaterThanOrEqualTo => lhs >= rhs,
			ComparisonOperator::LessThan => lhs < rhs,
			ComparisonOperator::LessThanOrEqualTo => lhs <= rhs,
			ComparisonOperator::NotEqual => lhs != rhs,
			ComparisonOperator::InArray => unimplemented!(),
			ComparisonOperator::NotInArray => unimplemented!(),
			ComparisonOperator::Exists => unimplemented!(),
			ComparisonOperator::Type => unimplemented!(),
		}
	}
	pub fn compare_json(
		&self,
		lhs: &serde_json::Value,
		rhs: &serde_json::Value,
	) -> bool {
		match (lhs, rhs) {
			(
				serde_json::Value::Number(lhs),
				serde_json::Value::Number(rhs),
			) => match self {
				ComparisonOperator::EqualTo => lhs == rhs,
				ComparisonOperator::GreaterThan => {
					lhs.as_f64().unwrap() > rhs.as_f64().unwrap()
				}
				ComparisonOperator::GreaterThanOrEqualTo => {
					lhs.as_f64().unwrap() >= rhs.as_f64().unwrap()
				}
				ComparisonOperator::LessThan => {
					lhs.as_f64().unwrap() < rhs.as_f64().unwrap()
				}
				ComparisonOperator::LessThanOrEqualTo => {
					lhs.as_f64().unwrap() <= rhs.as_f64().unwrap()
				}
				ComparisonOperator::NotEqual => lhs != rhs,
				ComparisonOperator::InArray => unimplemented!(),
				ComparisonOperator::NotInArray => unimplemented!(),
				ComparisonOperator::Exists => unimplemented!(),
				ComparisonOperator::Type => unimplemented!(),
			},
			(
				serde_json::Value::String(lhs),
				serde_json::Value::String(rhs),
			) => match self {
				ComparisonOperator::EqualTo => lhs == rhs,
				ComparisonOperator::GreaterThan => lhs > rhs,
				ComparisonOperator::GreaterThanOrEqualTo => lhs >= rhs,
				ComparisonOperator::LessThan => lhs < rhs,
				ComparisonOperator::LessThanOrEqualTo => lhs <= rhs,
				ComparisonOperator::NotEqual => lhs != rhs,
				ComparisonOperator::InArray => unimplemented!(),
				ComparisonOperator::NotInArray => unimplemented!(),
				ComparisonOperator::Exists => unimplemented!(),
				ComparisonOperator::Type => unimplemented!(),
			},
			_ => false,
		}
	}
}
