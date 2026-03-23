melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 f4 g4 bes4 c4 bes4 a4 g4 \break

  % Line 2
  g8 ees4 f4 g4 f8( ees8) d2 c2 \break

  % Line 3
  g'2 f4 g4 bes4 c4 bes4 a4 g4 \break

  % Line 4
  g8 ees4 f4 g4 f8( ees8) d2 c2 \break

  % Line 5
  c4 ees4 f4 c4 ees4 f4 g4 \break

  % Line 6
  g8 c4 bes4 c4 d4 bes4 a4 g4 \break

  % Line 7
  g8 bes4 g4 bes4 f4 ees4 d4 c2 \break

  % Line 8
  g'4 f8( ees8) d2 c1 \bar "|."
}
