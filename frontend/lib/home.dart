import 'dart:io';

import 'package:flutter/material.dart';
import 'package:frontend/ride.dart';
import 'package:http/http.dart';

class HomePage extends StatefulWidget {
  const HomePage({super.key, required this.title});

  final String title;

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  Ride ride = Ride(
    from: Address(
      street: 'Krucza 50',
      city: 'Warsaw',
      zip: '01-023',
    ),
    to: Address(
      street: 'Klonowa 24',
      city: 'Warsaw',
      zip: '05-077',
    ),
    id: 8765,
    state: RideState.accepted,
  );

  /*
  Future<Ride> getRide() async {
    var legacyAddress = Platform.environment['USVC_LEGACY'] ?? '';
    var legacyUri = Uri.parse(legacyAddress);
    var endpoint = legacyUri.resolve('/transits');
    var http = HttpClient();
    var req = await http.postUrl(endpoint);
    
  }
  */

  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    FloatingActionButton? fab;
    if (ride.state != RideState.finished) {
      fab = FloatingActionButton(
        onPressed: () {
          setState(() {
            ride.state = RideState.finished;
          });
        },
        tooltip: 'Finish drive',
        child: const Icon(Icons.ads_click),
      );
    }
    return Scaffold(
      appBar: bar(),
      body: RideWidget(ride: ride),
      floatingActionButton: fab,
    );
  }

  AppBar bar() {
    return AppBar(
      foregroundColor: Theme.of(context).colorScheme.onPrimary,
      backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      title: Text(widget.title),
      leading: IconButton(
        onPressed: () {},
        icon: const Icon(Icons.drive_eta),
      ),
    );
  }

  void _finishRide() {}
}
