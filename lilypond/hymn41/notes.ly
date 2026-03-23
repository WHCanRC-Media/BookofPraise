melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 f4 g4 f4 d'4 c8( d8) c4 bes4 \break

  % Line 2
  a4 g4 bes4 g4 f4 ees8( f8) d2 \break

  % Line 3
  f4 f4 g4 f4 d'4 c8( d8) c4 bes4 \break

  % Line 4
  g4 ees'4 d4 bes4 a4 g8( a8) bes2 \break

  % Line 5
  bes4 bes4 bes4 a4 g4 a8( g8) fis4 d4 \break

  % Line 6
  d'4 d4 ees4 d4 c4 c8( d8) c2 \break

  % Line 7
  f,4 f4 g4 f4 d'4 c8( d8) c4 bes4 \break

  % Line 8
  g4 ees'4 d4 bes4 a4 g8( a8) bes2 \bar "|."
}
