use rumqttc::{AsyncClient, QoS};

use super::BookingIndex;

async fn update_all(client: &AsyncClient, bookings: &BookingIndex) {
	let displays_to_update =
		bookings
		.iter()
		.filter(|(_machine, booking)|
			booking.is_running()
			&& minute_mark(booking.total_runtime())
		);

	for (machine, booking) in displays_to_update {
		update_one(client, machine, booking.total_runtime());
	}
		
}

async fn update_one(client: &AsyncClient, machine_id: &str, runtime: Duration) {
	let messages = [
		("title", "Dauer".into()),
		("info", create_display_time_string(runtime)),
	];

	for (route, payload) in messages {
		client
		.publish(
			format!("fabreader/{machine_id}/display/{route}"),
			QoS::AtMostOnce,
			false,
			payload
		)
		.await
		.expect("failed to publish display update");
	}
}

/// whether this duration crossed a minute boundary within the last second
const fn minute_mark(duration: Duration) -> bool {
    duration
	.as_secs()
	.is_multiple_of(60)
}

fn create_display_time_string(runtime: Duration) -> String {
    let mut total_minutes = runtime.as_secs() / 60;
    if !runtime.is_zero() {
        total_minutes += 1; // workaround so partial minutes get rounded up instead of down
    }

    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    format!("{hours:.0}:{minutes:0>2.0}")
}