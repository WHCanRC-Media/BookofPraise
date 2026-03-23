melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 bes4 a8( g8) f4 bes4 d,4 ees4 f4 \break

  % Line 2
  f4 g8( a8) bes4 c4 c4 d2 \break

  % Line 3
  f,4 bes4 a8( g8) f4 bes4 d,4 ees4 f4 \break

  % Line 4
  f4 g8( a8) bes4 bes4 a4 bes2 \break

  % Line 5
  bes8( c8) d4 c4 d4 ees4 c4 a4( bes4) c4 \break

  % Line 6
  bes8( c8) d4 c4 d4 ees4 c2 \break

  % Line 7
  f,4 bes4 a8( g8) f4 bes4 d,4 ees4 f4 \break

  % Line 8
  f4 g8( a8) bes4 bes4 a4 bes2 \bar "|."
}
