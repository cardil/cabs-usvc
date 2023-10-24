import 'package:flutter/material.dart';

class Address {
  final String? country;
  final String? district;
  final String city;
  final String street;
  final String zip;

  Address({
    required this.city,
    required this.street,
    required this.zip,
    this.country,
    this.district
  });
}

enum RideState {
  published,
  accepted,
  inTransit,
  finished,
}

class Ride {
  final Address from;
  final Address to;
  final int id;
  RideState state;

  Ride({
    required this.from,
    required this.to,
    required this.id,
    required this.state,
  });
}

class RideWidget extends StatelessWidget {
  final Ride ride;

  const RideWidget({required this.ride, super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      children: <Widget>[
        header(context: context, id: ride.id, state: ride.state),
        adressLine(context: context, address: ride.from, label: 'From'),
        adressLine(context: context, address: ride.to, label: 'To'),
      ],
    );
  }

  Container header({
    required BuildContext context,
    required int id,
    required RideState state,
  }) {
    String state = '<< on-going >>';
    TextStyle stateStyle = Theme.of(context).textTheme.titleSmall!;
    TextStyle titleStyle = Theme.of(context).textTheme.headlineMedium!;
    Color? iconColor;
    Color finishedColor = Colors.green;
    if (ride.state == RideState.finished) {
      state = 'finished';
      stateStyle = stateStyle.copyWith(color: finishedColor);
      titleStyle = titleStyle.copyWith(color: finishedColor);
      iconColor = finishedColor;
    }
    return Container(
      padding: const EdgeInsets.all(20),
      child: Column(children: <Widget>[
        Icon(Icons.directions_car, size: 60, color: iconColor),
        Text('Ride #${ride.id}', style: titleStyle),
        Text(state, style: stateStyle),
      ]),
    );
  }

  Container adressLine({
    required BuildContext context,
    required Address address,
    required String label,
  }) {
    return Container(
      padding: const EdgeInsets.all(10),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: <Widget>[
          ConstrainedBox(
            constraints: const BoxConstraints(minWidth: 80),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: <Widget>[
                const Icon(Icons.location_on),
                Text(label, style: Theme.of(context).textTheme.titleMedium),
              ],
            ),
          ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Text(address.street,
                    style: Theme.of(context).textTheme.headlineMedium),
                Text('${address.zip} ${address.city}'),
              ],
            ),
          )
        ],
      ),
    );
  }
}
