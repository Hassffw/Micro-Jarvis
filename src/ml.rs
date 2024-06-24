use linfa::prelude::*;
use linfa_linear::{LinearRegression, FittedLinearRegression};
use ndarray::{Array1, Array2};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableLinearModel {
    coefficients: Vec<f64>,
    intercept: f64,
}

impl SerializableLinearModel {
    pub fn from_fitted(model: &FittedLinearRegression<f64>) -> Self {
        SerializableLinearModel {
            coefficients: model.params().to_vec(),
            intercept: model.intercept(),
        }
    }

    pub fn to_linear_regression(&self) -> LinearRegression {
        LinearRegression::new()
            .with_intercept(true)
    }
}

pub fn train_model(x: &Array2<f64>, y: &Array1<f64>) -> Result<SerializableLinearModel, Box<dyn std::error::Error>> {
    let dataset = Dataset::new(x.clone(), y.clone());
    let model = LinearRegression::new()
        .with_intercept(true)
        .fit(&dataset)?;
    Ok(SerializableLinearModel::from_fitted(&model))
}

pub fn predict(model: &SerializableLinearModel, x: &Array2<f64>) -> Array1<f64> {
    let lr = model.to_linear_regression();
    let dataset = Dataset::new(x.clone(), Array1::zeros(x.nrows()));
    let fitted_model = lr.fit(&dataset).unwrap();
    fitted_model.predict(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_serialization_deserialization() {
        let x = Array2::from_shape_vec((3, 2), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let model = train_model(&x, &y).unwrap();
        let serialized = serde_json::to_string(&model).unwrap();
        let deserialized: SerializableLinearModel = serde_json::from_str(&serialized).unwrap();

        assert_abs_diff_eq!(model.coefficients, deserialized.coefficients, epsilon = 1e-10);
        assert_abs_diff_eq!(model.intercept, deserialized.intercept, epsilon = 1e-10);

        let predictions = predict(&model, &x);
        let deserialized_predictions = predict(&deserialized, &x);

        assert_abs_diff_eq!(predictions, deserialized_predictions, epsilon = 1e-10);
    }
}
